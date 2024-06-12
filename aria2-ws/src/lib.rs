//!
//! An aria2 websocket jsonrpc in Rust.
//!
//! [aria2 RPC docs](https://aria2.github.io/manual/en/html/aria2c.html#methods)
//!
//! ## Features
//!
//! - Almost all methods and structed responses
//! - Auto reconnect
//! - Ensures `on_complete` and `on_error` hook to be executed even after reconnected.
//! - Supports notifications
//!
//! ## Example
//!
//! ```no_run
//! use std::sync::Arc;
//!
//! use aria2_ws::{Client, Callbacks, TaskOptions};
//! use futures::FutureExt;
//! use serde_json::json;
//! use tokio::{spawn, sync::Semaphore};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::connect("ws://127.0.0.1:6800/jsonrpc", None)
//!         .await
//!         .unwrap();
//!     let options = TaskOptions {
//!         split: Some(2),
//!         header: Some(vec!["Referer: https://www.pixiv.net/".to_string()]),
//!         all_proxy: Some("http://127.0.0.1:10809".to_string()),
//!         // Add extra options which are not included in TaskOptions.
//!         extra_options: json!({"max-download-limit": "200K"})
//!             .as_object()
//!             .unwrap()
//!             .clone(),
//!         ..Default::default()
//!     };
//!
//!     // use `tokio::sync::Semaphore` to wait for all tasks to finish.
//!     let semaphore = Arc::new(Semaphore::new(0));
//!     client
//!         .add_uri(
//!             vec![
//!                 "https://i.pximg.net/img-original/img/2020/05/15/06/56/03/81572512_p0.png"
//!                     .to_string(),
//!             ],
//!             Some(options.clone()),
//!             None,
//!             Some(Callbacks {
//!                 on_complete: Some({
//!                     let s = semaphore.clone();
//!                     async move {
//!                         s.add_permits(1);
//!                         println!("Task 1 completed!");
//!                     }
//!                     .boxed()
//!                 }),
//!                 on_error: Some({
//!                     let s = semaphore.clone();
//!                     async move {
//!                         s.add_permits(1);
//!                         println!("Task 1 error!");
//!                     }
//!                     .boxed()
//!                 }),
//!             }),
//!         )
//!         .await
//!         .unwrap();
//!
//!     // Will 404
//!     client
//!         .add_uri(
//!             vec![
//!                 "https://i.pximg.net/img-original/img/2022/01/05/23/32/16/95326322_p0.pngxxxx"
//!                     .to_string(),
//!             ],
//!             Some(options.clone()),
//!             None,
//!             Some(Callbacks {
//!                 on_complete: Some({
//!                     let s = semaphore.clone();
//!                     async move {
//!                         s.add_permits(1);
//!                         println!("Task 2 completed!");
//!                     }
//!                     .boxed()
//!                 }),
//!                 on_error: Some({
//!                     let s = semaphore.clone();
//!                     async move {
//!                         s.add_permits(1);
//!                         println!("Task 2 error!");
//!                     }
//!                     .boxed()
//!                 }),
//!             }),
//!         )
//!         .await
//!         .unwrap();
//!
//!     let mut not = client.subscribe_notifications();
//!
//!     spawn(async move {
//!         while let Ok(msg) = not.recv().await {
//!             println!("Received notification {:?}", &msg);
//!         }
//!     });
//!
//!     // Wait for 2 tasks to finish.
//!     let _ = semaphore.acquire_many(2).await.unwrap();
//!
//!     client.shutdown().await.unwrap();
//! }
//!
//! ```

mod callback;
mod client;
mod error;
mod method;
mod options;
pub mod response;
mod utils;

pub use error::Error;
pub use options::TaskOptions;
// Re-export `Map` for `TaskOptions`.
pub use callback::Callbacks;
pub use client::{Client, InnerClient};
pub use serde_json::Map;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::OptionExt;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub id: Option<i32>,
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Vec<Value>,
}

/// Error returned by RPC calls.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aria2Error {
    pub code: i32,
    pub message: String,
}
impl std::fmt::Display for Aria2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "aria2 responsed error: code {}: {}",
            self.code, self.message
        )
    }
}
impl std::error::Error for Aria2Error {}

#[derive(Deserialize, Debug, Clone)]
pub struct RpcResponse {
    pub id: Option<i32>,
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<Aria2Error>,
}

/// Events about download progress from aria2.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Event {
    Start,
    Pause,
    Stop,
    Complete,
    Error,
    /// This notification will be sent when a torrent download is complete but seeding is still going on.
    BtComplete,
}

impl TryFrom<&str> for Event {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self> {
        use Event::*;
        let event = match value {
            "aria2.onDownloadStart" => Start,
            "aria2.onDownloadPause" => Pause,
            "aria2.onDownloadStop" => Stop,
            "aria2.onDownloadComplete" => Complete,
            "aria2.onDownloadError" => Error,
            "aria2.onBtDownloadComplete" => BtComplete,
            _ => return error::ParseSnafu { value, to: "Event" }.fail(),
        };
        Ok(event)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    Aria2 { gid: String, event: Event },
    WebSocketConnected,
    WebsocketClosed,
}

impl TryFrom<&RpcRequest> for Notification {
    type Error = crate::Error;

    fn try_from(req: &RpcRequest) -> Result<Self> {
        let gid = (|| req.params.get(0)?.get("gid")?.as_str())()
            .with_context(|| error::ParseSnafu {
                value: format!("{:?}", req),
                to: "Notification",
            })?
            .to_string();
        let event = req.method.as_str().try_into()?;
        Ok(Notification::Aria2 { gid, event })
    }
}

#[cfg(test)]
mod tests {
    fn check_if_send<T: Send + Sync>() {}

    #[test]
    fn t() {
        check_if_send::<crate::error::Error>();
        check_if_send::<crate::Client>();
    }
}
