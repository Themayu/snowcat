use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use http::Request;
use rustls::ClientConfig;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
	Connector,
	MaybeTlsStream,
	WebSocketStream,
	connect_async_tls_with_config,
};
use tungstenite::Message;
use tungstenite::error::Error as TungsteniteError;
use tungstenite::protocol::WebSocketConfig;

pub struct Connection {
	reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
	writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

const SOCKET_CONFIG: WebSocketConfig = WebSocketConfig {
	max_send_queue: None,
	max_message_size: Some(64 << 20),
	max_frame_size: Some(16 << 20),
	accept_unmasked_frames: false,
};

impl Connection {
	/// Try to open a new websocket connection to the F-Chat server.
	pub async fn open(request: Request<()>, rustls_config: Arc<ClientConfig>) -> Result<Self, ConnectionError> {
		// ignore the response object; we have a stream and that's all we care
		// about.
		let (stream, _) = connect_async_tls_with_config(
			request,
			Some(SOCKET_CONFIG),
			Some(Connector::Rustls(rustls_config))
		).await?;

		let (writer, reader) = stream.split();

		Ok(Connection {
			reader,
			writer,
		})
	}

	pub async fn close(self) -> Result<(), TungsteniteError> {
		let Connection { reader, writer, .. } = self;

		let mut stream = reader.reunite(writer).expect("reader and writer should be from the same socket");

		stream.close(None).await
	}
}

#[derive(Debug, Error)]
pub enum ConnectionError {
	#[error("{0}")]
	Io(#[from] std::io::Error),

	#[error("{0}")]
	Tungstenite(#[from] TungsteniteError),
}
