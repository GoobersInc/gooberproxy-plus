use anyhow::Result;
use azalea_protocol::packets::game::{
    serverbound_keep_alive_packet::ServerboundKeepAlivePacket, ClientboundGamePacket,
};

use crate::{conn::ClientGameConn, App};

impl App {
    /// Takes a `ClientGameConn` and responds to keep-alive packets
    pub async fn keep_alive(&self, conn: &mut ClientGameConn) -> Result<()> {
        loop {
            if let ClientboundGamePacket::KeepAlive(packet) = conn.read().await? {
                let packet = ServerboundKeepAlivePacket { id: packet.id };
                conn.write(packet.get()).await?;
            }
        }
    }
}
