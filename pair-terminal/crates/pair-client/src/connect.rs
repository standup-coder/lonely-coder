use pair_common::protocol::*;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn connect(server_url: &str) -> anyhow::Result<WsStream> {
    let (ws_stream, _) = connect_async(server_url).await?;
    Ok(ws_stream)
}

pub fn serialize_message(msg: &ClientMessage) -> anyhow::Result<String> {
    Ok(serde_json::to_string(msg)?)
}

#[allow(dead_code)]
pub fn deserialize_server_message(data: &str) -> anyhow::Result<ServerMessage> {
    Ok(serde_json::from_str(data)?)
}
