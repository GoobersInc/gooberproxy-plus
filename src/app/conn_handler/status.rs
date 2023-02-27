use anyhow::{bail as yeet, Context, Result};
use azalea_chat::{text_component::TextComponent, FormattedText};
use azalea_protocol::packets::{
    status::{
        clientbound_pong_response_packet::ClientboundPongResponsePacket,
        clientbound_status_response_packet::{
            ClientboundStatusResponsePacket, Players as StatusPlayers, Version as StatusVersion,
        },
        ServerboundStatusPacket,
    },
    PROTOCOL_VERSION,
};
use tracing::info;

use crate::{app::App, conn::ServerStatusConn};

impl App {
    /// Handles a client that has specified it wants to receive a status
    /// response (server list ping)
    pub async fn handle_status(&self, mut conn: ServerStatusConn) -> Result<()> {
        info!("Handling status request");

        // Read the request
        let _ = match conn.read().await.context("Failed to read status request")? {
            ServerboundStatusPacket::StatusRequest(request) => request,
            _ => {
                yeet!("Expected status request");
            }
        };

        // Send the response
        let status_response = ClientboundStatusResponsePacket {
            description: FormattedText::Text(TextComponent::new(
                "Goobers Inc. secret test server (real)".to_string(),
            )),
            favicon: None,
            players: StatusPlayers {
                max: 420,
                online: 69,
                sample: vec![],
            },
            version: StatusVersion {
                name: "popbob sex dupe 1.69.4".to_string(),
                protocol: PROTOCOL_VERSION as i32,
            },
            previews_chat: None,
            enforces_secure_chat: None,
        };
        conn.write(status_response.get())
            .await
            .context("Failed to write status response")?;

        // Read the request
        let ping_request = match conn.read().await.context("Failed to read ping request")? {
            ServerboundStatusPacket::PingRequest(ping_request) => ping_request,
            _ => {
                yeet!("Expected ping request");
            }
        };

        // Send the response
        let ping_response = ClientboundPongResponsePacket {
            time: ping_request.time,
        };
        conn.write(ping_response.get())
            .await
            .context("Failed to write pong response")?;

        Ok(())
    }
}
