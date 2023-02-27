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
                    // Get the remote address
                    let remote = match socket.peer_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            // ??? what
                            error!("Failed to get peer address: {}", e);
                            continue;
                        }
                    };

                    // Create a fancy logging context
                    let span = span!(Level::TRACE, "Connection", remote = %remote);

                    // Yeet it off a cliff to teach it how to fly
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
                    // Again, ??? what
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
