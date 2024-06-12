use snafu::prelude::*;
use tokio_tungstenite::tungstenite::Error as WsError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("aria2: responsed error: {source}"))]
    Aria2 { source: crate::Aria2Error },
    #[snafu(display("aria2: cannot parse value {value:?} as {to}"))]
    Parse { value: String, to: String },
    #[snafu(display("aria2: websocket error: {source}"))]
    WebsocketIo { source: WsError },
    #[snafu(display("aria2: json error: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("aria2: websocket closed: {message}"))]
    WebsocketClosed { message: String },
    #[snafu(display("aria2: reconnect task timeout"))]
    ReconnectTaskTimeout { source: tokio::time::error::Elapsed },
}
