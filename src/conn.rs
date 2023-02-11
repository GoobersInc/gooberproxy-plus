#![allow(dead_code)]

use azalea_protocol::{
    connect::Connection,
    packets::{
        game::{ClientboundGamePacket, ServerboundGamePacket},
        handshake::{ClientboundHandshakePacket, ServerboundHandshakePacket},
        login::{ClientboundLoginPacket, ServerboundLoginPacket},
        status::{ClientboundStatusPacket, ServerboundStatusPacket},
    },
};

pub type ServerHandshakeConn = Connection<ServerboundHandshakePacket, ClientboundHandshakePacket>;
pub type ServerStatusConn = Connection<ServerboundStatusPacket, ClientboundStatusPacket>;
pub type ServerLoginConn = Connection<ServerboundLoginPacket, ClientboundLoginPacket>;
pub type ServerGameConn = Connection<ServerboundGamePacket, ClientboundGamePacket>;

pub type ClientHandshakeConn = Connection<ClientboundHandshakePacket, ServerboundHandshakePacket>;
pub type ClientStatusConn = Connection<ClientboundStatusPacket, ServerboundStatusPacket>;
pub type ClientLoginConn = Connection<ClientboundLoginPacket, ServerboundLoginPacket>;
pub type ClientGameConn = Connection<ClientboundGamePacket, ServerboundGamePacket>;
