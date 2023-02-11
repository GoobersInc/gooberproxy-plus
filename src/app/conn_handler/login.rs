use anyhow::{bail as yeet, Context, Result};
use azalea_chat::{text_component::TextComponent, FormattedText};
use azalea_protocol::packets::login::{
    clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket, ServerboundLoginPacket,
};
use tracing::debug;

use crate::{app::App, conn::ServerLoginConn};

impl App {
    pub async fn handle_login(&self, mut conn: ServerLoginConn) -> Result<()> {
        debug!("Handling login request");

        let hello = match conn.read().await.context("Failed to read login request")? {
            ServerboundLoginPacket::Hello(hello) => hello,
            _ => {
                yeet!("Expected hello");
            }
        };
        debug!("Hello: {:?}", hello);

        let kick = ClientboundLoginDisconnectPacket {
            reason: FormattedText::Text(TextComponent::new("goober".to_string())),
        };
        conn.write(kick.get())
            .await
            .context("Failed to write kick")?;

        Ok(())
    }
}
