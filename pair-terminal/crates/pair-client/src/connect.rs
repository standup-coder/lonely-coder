use pair_common::protocol::*;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async_tls, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect(server_url: &str) -> anyhow::Result<WsStream> {
    let (ws_stream, _) = connect_async_tls(server_url).await?;
    Ok(ws_stream)
}

pub fn serialize_message(msg: &ClientMessage) -> anyhow::Result<String> {
    Ok(serde_json::to_string(msg)?)
}

pub fn deserialize_server_message(data: &str) -> anyhow::Result<ServerMessage> {
    Ok(serde_json::from_str(data)?)
}
