use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{error, span, Level};
use tracing_futures::Instrument;

use crate::app::App;

impl App {
    pub async fn listen_for_connections(&self, listener: TcpListener) -> Result<()> {
        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    let remote = match socket.peer_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            error!("Failed to get peer address: {}", e);
                            continue;
                        }
                    };
                    let span = span!(Level::TRACE, "Connection", remote = %remote);
                    let clone = self.clone();
                    tokio::spawn(
                        async move {
                            if let Err(e) = clone.handle_connection(socket).await {
                                error!("Error while handling connection: {}", e);
                            }
                        }
                        .instrument(span),
                    );
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
