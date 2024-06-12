use crate::{
    callback::{callback_worker, TaskCallbacks},
    error,
    utils::print_error,
    Callbacks, Notification, Result, RpcRequest, RpcResponse,
};
use futures::prelude::*;
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde_json::Value;
use snafu::prelude::*;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    select, spawn,
    sync::{broadcast, mpsc, oneshot, Notify},
    time::sleep,
};
use tokio_tungstenite::tungstenite::Message;
type WebSocket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug)]
pub(crate) struct Subscription {
    pub id: i32,
    pub tx: oneshot::Sender<RpcResponse>,
}
pub struct InnerClient {
    token: Option<String>,
    id: AtomicI32,
    /// Channel for sending messages to the websocket.
    tx_ws_sink: mpsc::Sender<Message>,
    tx_notification: broadcast::Sender<Notification>,
    tx_subscription: mpsc::Sender<Subscription>,
    /// On notified, all spawned tasks shut down.
    shutdown: Arc<Notify>,
}

/// An aria2 websocket rpc client.
///
/// # Example
///
/// ```
/// use aria2_ws::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::connect("ws://127.0.0.1:6800/jsonrpc", None)
///         .await
///         .unwrap();
///     let version = client.get_version().await.unwrap();
///     println!("{:?}", version);
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    inner: Arc<InnerClient>,
    // The sender can be cloned like `Arc`.
    tx_callback: mpsc::UnboundedSender<TaskCallbacks>,
}

impl Drop for InnerClient {
    fn drop(&mut self) {
        // notify all spawned tasks to shutdown
        debug!("InnerClient dropped, notify shutdown");
        self.shutdown.notify_waiters();
    }
}

async fn process_ws(
    ws: WebSocket,
    rx_ws_sink: &mut mpsc::Receiver<Message>,
    tx_notification: broadcast::Sender<Notification>,
    rx_subscription: &mut mpsc::Receiver<Subscription>,
) {
    let (mut sink, mut stream) = ws.split();
    let mut subscriptions = HashMap::<i32, oneshot::Sender<RpcResponse>>::new();

    let on_stream = |msg: String,
                     subscriptions: &mut HashMap<i32, oneshot::Sender<RpcResponse>>|
     -> Result<()> {
        let v: Value = serde_json::from_str(&msg).context(error::JsonSnafu)?;
        if let Value::Object(obj) = &v {
            if obj.contains_key("method") {
                // The message should be a notification.
                // https://aria2.github.io/manual/en/html/aria2c.html#notifications
                let req: RpcRequest = serde_json::from_value(v).context(error::JsonSnafu)?;
                let notification = (&req).try_into()?;
                let _ = tx_notification.send(notification);
                return Ok(());
            }
        }

        // The message should be a response.
        let res: RpcResponse = serde_json::from_value(v).context(error::JsonSnafu)?;
        if let Some(ref id) = res.id {
            let tx = subscriptions.remove(id);
            if let Some(tx) = tx {
                let _ = tx.send(res);
            }
        }
        Ok(())
    };

    loop {
        select! {
            msg = stream.try_next() => {
                debug!("websocket received message: {:?}", msg);
                let Ok(msg) = msg else {
                    break;
                };
                if let Some(Message::Text(s)) = msg {
                    print_error(on_stream(s, &mut subscriptions));
                }
            },
            msg = rx_ws_sink.recv() => {
                debug!("writing message to websocket: {:?}", msg);
                let Some(msg) = msg else {
                    break;
                };
                print_error(sink.send(msg).await);
            },
            subscription = rx_subscription.recv() => {
                if let Some(subscription) = subscription {
                    subscriptions.insert(subscription.id, subscription.tx);
                }
            }
        }
    }
}

impl InnerClient {
    pub(crate) async fn connect(url: &str, token: Option<&str>) -> Result<Self> {
        let (tx_ws_sink, mut rx_ws_sink) = mpsc::channel(1);
        let (tx_subscription, mut rx_subscription) = mpsc::channel(1);
        let shutdown = Arc::new(Notify::new());
        // Broadcast notifications to all subscribers.
        // The receiver is dropped cause there is no subscriber for now.
        let (tx_notification, _) = broadcast::channel(1);

        let inner = InnerClient {
            tx_ws_sink,
            id: AtomicI32::new(0),
            token: token.map(|t| "token:".to_string() + t),
            tx_subscription,
            tx_notification: tx_notification.clone(),
            shutdown: shutdown.clone(),
        };

        async fn connect_ws(url: &str) -> Result<WebSocket> {
            debug!("connecting to {}", url);
            let (ws, res) = tokio_tungstenite::connect_async(url)
                .await
                .context(error::WebsocketIoSnafu)?;
            debug!("connected to {}, {:?}", url, res);
            Ok(ws)
        }

        let ws = connect_ws(url).await?;
        let url = url.to_string();
        // spawn a task to process websocket messages
        spawn(async move {
            let mut ws = Some(ws);
            loop {
                if let Some(ws) = ws.take() {
                    let _ = tx_notification.send(Notification::WebSocketConnected);

                    let fut = process_ws(
                        ws,
                        &mut rx_ws_sink,
                        tx_notification.clone(),
                        &mut rx_subscription,
                    );

                    select! {
                        _ = fut => {},
                        _ = shutdown.notified() => {
                            return;
                        },
                    }

                    let _ = tx_notification.send(Notification::WebsocketClosed);
                } else {
                    let r = select! {
                        r = connect_ws(&url) => r,
                        _ = shutdown.notified() => return,
                    };
                    match r {
                        Ok(ws_) => {
                            ws.replace(ws_);
                        }
                        Err(err) => {
                            info!("{}", err);
                            sleep(Duration::from_secs(3)).await;
                        }
                    }
                }
            }
        });

        Ok(inner)
    }

    fn id(&self) -> i32 {
        self.id.fetch_add(1, Ordering::Relaxed)
    }

    async fn wait_for_id<T>(&self, id: i32, rx: oneshot::Receiver<RpcResponse>) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        let res = rx.await.map_err(|err| {
            error::WebsocketClosedSnafu {
                message: format!("receiving response for id {}: {}", id, err),
            }
            .build()
        })?;

        if let Some(err) = res.error {
            return Err(err).context(error::Aria2Snafu);
        }

        if let Some(v) = res.result {
            Ok(serde_json::from_value::<T>(v).context(error::JsonSnafu)?)
        } else {
            error::ParseSnafu {
                value: format!("{:?}", res),
                to: "RpcResponse with result",
            }
            .fail()
        }
    }

    /// Send a rpc request to websocket without waiting for response.
    pub async fn call(&self, id: i32, method: &str, mut params: Vec<Value>) -> Result<()> {
        if let Some(ref token) = self.token {
            params.insert(0, Value::String(token.clone()))
        }
        let req = RpcRequest {
            id: Some(id),
            jsonrpc: "2.0".to_string(),
            method: "aria2.".to_string() + method,
            params,
        };
        self.tx_ws_sink
            .send(Message::Text(
                serde_json::to_string(&req).context(error::JsonSnafu)?,
            ))
            .await
            .expect("tx_ws_sink: receiver has been dropped");
        Ok(())
    }

    /// Send a rpc request to websocket and wait for corresponding response.
    pub async fn call_and_wait<T>(&self, method: &str, params: Vec<Value>) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        let id = self.id();
        let (tx, rx) = oneshot::channel();
        self.tx_subscription
            .send(Subscription { id, tx })
            .await
            .expect("tx_subscription: receiver has been closed");

        self.call(id, method, params).await?;
        self.wait_for_id::<T>(id, rx).await
    }

    /// Subscribe to notifications.
    ///
    /// Returns a instance of `broadcast::Receiver` which can be used to receive notifications.
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<Notification> {
        self.tx_notification.subscribe()
    }
}

impl Client {
    /// Create a new `Client` that connects to the given url.
    ///
    /// # Example
    ///
    /// ```
    /// use aria2_ws::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::connect("ws://127.0.0.1:6800/jsonrpc", None)
    ///         .await
    ///         .unwrap();
    ///     let gid = client
    ///         .add_uri(
    ///             vec!["https://go.dev/dl/go1.17.6.windows-amd64.msi".to_string()],
    ///             None,
    ///             None,
    ///             None,
    ///         )
    ///         .await
    ///         .unwrap();
    ///     client.force_remove(gid).await.unwrap();
    /// }
    /// ```
    pub async fn connect(url: &str, token: Option<&str>) -> Result<Self> {
        let inner = Arc::new(InnerClient::connect(url, token).await?);

        let weak = Arc::downgrade(&inner);
        let rx_notification = inner.subscribe_notifications();
        let (tx_callback, rx_callback) = mpsc::unbounded_channel();
        // hold a weak reference to `inner` to prevent not shutting down when `Client` is dropped
        spawn(callback_worker(weak, rx_notification, rx_callback));

        Ok(Self { inner, tx_callback })
    }

    pub(crate) fn add_callbacks(&self, gid: String, callbacks: Callbacks) {
        self.tx_callback
            .send(TaskCallbacks { gid, callbacks })
            .expect("tx_callback: receiver has been dropped");
    }
}

impl Deref for Client {
    type Target = InnerClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
