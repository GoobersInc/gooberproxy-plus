mod login;
mod status;

use anyhow::{bail as yeet, Context, Result};
use azalea_protocol::{
    connect::Connection,
    packets::{handshake::ServerboundHandshakePacket, ConnectionProtocol as HandshakeIntention},
};
use tokio::net::TcpStream;
use tracing::{debug, info};

use crate::{app::App, conn::ServerHandshakeConn};

impl App {
    pub async fn handle_connection(&self, socket: TcpStream) -> Result<()> {
        socket.nodelay()?;

        info!("Accepted connection");

        let mut conn: ServerHandshakeConn = Connection::wrap(socket);

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
                // yeet!("Unsupported intention: {:?}", intention);
                yeet!("Unsupported intention: {intention:?}");
            }
        }

        Ok(())
    }
}
