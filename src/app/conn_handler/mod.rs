use anyhow::{bail as yeet, Context, Result};
use azalea_protocol::{
    connect::Connection,
    packets::{handshake::ServerboundHandshakePacket, ConnectionProtocol as HandshakeIntention},
};
use tokio::net::TcpStream;
use tracing::{debug, info};

use crate::{app::App, conn::ServerHandshakeConn};

mod login;
mod status;

impl App {
    /// Accepts a TCP stream, determines what to do with it and does it
    pub async fn handle_connection(&self, socket: TcpStream) -> Result<()> {
        socket.nodelay()?;

        info!("Accepted connection");

        // This is a laughing matter
        let mut conn: ServerHandshakeConn = Connection::wrap(socket);

        // Read the handshake, determine what to do with it and do it
        let ServerboundHandshakePacket::ClientIntention(handshake) =
            conn.read().await.context("Failed to read handshake")?;
        debug!("Handshake: {:?}", handshake);
        match handshake.intention {
            HandshakeIntention::Status => {
                self.handle_status(Connection::from(conn)).await?;
            }
            HandshakeIntention::Login => {
                self.handle_login(Connection::from(conn)).await?;
            }
            intention => {
                // We can use Anyhow since this is only used for logging
                yeet!("Unsupported intention: {intention:?}");
            }
        }

        Ok(())
    }
}
