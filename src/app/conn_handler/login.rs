use anyhow::{bail as yeet, Context, Result};
use azalea_chat::{text_component::TextComponent, FormattedText};
use azalea_client::Account;
use azalea_protocol::packets::login::{
    clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket, ServerboundLoginPacket,
};
use tracing::{debug, info, warn};

use crate::{
    app::App,
    conn::ServerLoginConn,
    join::{join_server, say_hello},
};

impl App {
    pub async fn handle_login(&self, mut conn1: ServerLoginConn) -> Result<()> {
        info!("Handling login request");

        let hello = match conn1.read().await.context("Failed to read login request")? {
            ServerboundLoginPacket::Hello(hello) => hello,
            _ => {
                yeet!("Expected hello");
            }
        };
        debug!("Hello: {:?}", hello);

        if hello.username != self.config.player {
            warn!("Kicking unknown player {}", hello.username);
            let kick_packet = ClientboundLoginDisconnectPacket {
                reason: FormattedText::Text(TextComponent::new(String::from("goober"))),
            };
            conn1.write(kick_packet.get()).await?;
            return Ok(());
        }

        let conn1 = conn1.unwrap()?; // ← will not panic

        let conn2 = say_hello(&self.config.server_addr, hello).await?.unwrap()?; // ← will not panic

        let (mut conn1_read, mut conn1_write) = tokio::io::split(conn1);
        let (mut conn2_read, mut conn2_write) = tokio::io::split(conn2);

        info!("Relaying traffic");

        let who_disconnected = tokio::select! {
            _ = tokio::io::copy(&mut conn1_read, &mut conn2_write) => WhoDisconnected::Client,
            _ = tokio::io::copy(&mut conn2_read, &mut conn1_write) => WhoDisconnected::Server,
        };

        info!("{who_disconnected:?} disconnected");

        if who_disconnected == WhoDisconnected::Client {
            let account = Account::microsoft(&self.config.account).await?;
            let (mut conn, profile) = join_server(&self.config.server_addr, &account).await?;

            info!("Successfully reconnected as {}", profile.name);

            let app_clone = self.clone();
            tokio::spawn(async move {
                if let Err(err) = app_clone.keep_alive(&mut conn).await {
                    warn!("Keep alive task disconnected: {err}");
                }
            });
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WhoDisconnected {
    Client,
    Server,
}
