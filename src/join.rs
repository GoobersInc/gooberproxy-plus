use anyhow::{anyhow, Result};
use azalea::Account;
use azalea_auth::{game_profile::GameProfile, sessionserver::ClientSessionServerError};
use azalea_chat::FormattedText;
use azalea_protocol::{
    connect::{Connection, ConnectionError},
    packets::{
        handshake::client_intention_packet::ClientIntentionPacket,
        login::{
            serverbound_hello_packet::ServerboundHelloPacket,
            serverbound_key_packet::{NonceOrSaltSignature, ServerboundKeyPacket},
            ClientboundLoginPacket,
        },
        ConnectionProtocol, PROTOCOL_VERSION,
    },
    read::ReadPacketError,
};
use std::net::SocketAddr;
use thiserror::Error;

use crate::conn::{ClientGameConn, ClientLoginConn};

#[derive(Error, Debug)]
pub enum JoinServerError {
    #[error("Disconnected: {0}")]
    Disconnected(FormattedText),

    #[error(transparent)]
    Connection(#[from] ConnectionError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ReadPacket(#[from] Box<ReadPacketError>),

    #[error(transparent)]
    SessionServer(#[from] ClientSessionServerError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Start a connection, authenticate and join the server
/// without sending the encryption response
pub async fn almost_join_server(
    addr: &SocketAddr,
    account: &Account,
) -> Result<(ClientLoginConn, ServerboundKeyPacket, [u8; 16]), JoinServerError> {
    if account.access_token.is_none() || account.uuid.is_none() {
        return Err(anyhow!("Account needs to have both an access token and UUID").into());
    }

    let mut conn = Connection::new(addr).await?;

    // Handshake
    conn.write(
        ClientIntentionPacket {
            protocol_version: PROTOCOL_VERSION,
            hostname: addr.ip().to_string(),
            port: addr.port(),
            intention: ConnectionProtocol::Login,
        }
        .get(),
    )
    .await?;

    // login
    let mut conn = conn.login();
    conn.write(
        ServerboundHelloPacket {
            username: account.username.clone(),
            public_key: None,
            profile_id: None,
        }
        .get(),
    )
    .await?;

    let encryption_request = match conn.read().await? {
        ClientboundLoginPacket::Hello(packet) => packet,
        ClientboundLoginPacket::LoginDisconnect(packet) => {
            return Err(anyhow!(format!("Disconnected: {}", packet.reason)).into());
        }
        packet => {
            return Err(anyhow!("Unexpected packet:\n{:#?}", packet).into());
        }
    };

    // Lettuce do le auth (auth so goofy)
    let e =
        azalea_crypto::encrypt(&encryption_request.public_key, &encryption_request.nonce).unwrap();

    let secret_key = e.secret_key.to_owned();

    let access_token = { account.access_token.as_ref().unwrap().lock().to_owned() };
    conn.authenticate(
        &access_token, // it's safe here because we've already checked
        &account.uuid.unwrap(),
        secret_key, // but I clone it here ._.
        &encryption_request,
    )
    .await?;

    Ok((
        conn,
        ServerboundKeyPacket {
            key_bytes: e.encrypted_public_key,
            nonce_or_salt_signature: NonceOrSaltSignature::Nonce(e.encrypted_nonce),
        },
        secret_key,
    ))
}

/// Finish joining the server by sending the encryption response
/// and waiting for the game profile
pub async fn finish_joining_server(
    mut conn: ClientLoginConn,
    packet: ServerboundKeyPacket,
    sk: [u8; 16],
) -> Result<(ClientGameConn, GameProfile), JoinServerError> {
    conn.write(packet.get()).await?;
    conn.set_encryption_key(sk);

    loop {
        match conn.read().await? {
            ClientboundLoginPacket::GameProfile(p) => {
                return Ok((conn.game(), p.game_profile));
            }
            ClientboundLoginPacket::LoginCompression(p) => {
                conn.set_compression_threshold(p.compression_threshold);
            }
            ClientboundLoginPacket::LoginDisconnect(p) => {
                return Err(JoinServerError::Disconnected(p.reason));
            }
            p => {
                return Err(anyhow!("Unexpected packet:\n{:#?}", p).into());
            }
        }
    }
}

/// Start a connection, authenticate and join the server
pub async fn join_server(
    addr: &SocketAddr,
    account: &Account,
) -> Result<(ClientGameConn, GameProfile), JoinServerError> {
    let (conn, packet, sk) = almost_join_server(addr, account).await?;
    finish_joining_server(conn, packet, sk).await
}

pub async fn say_hello(
    addr: &SocketAddr,
    hello: ServerboundHelloPacket,
) -> Result<ClientLoginConn, JoinServerError> {
    let mut conn = Connection::new(addr).await?;

    // Handshake
    conn.write(
        ClientIntentionPacket {
            protocol_version: PROTOCOL_VERSION,
            hostname: addr.ip().to_string(),
            port: addr.port(),
            intention: ConnectionProtocol::Login,
        }
        .get(),
    )
    .await?;

    let mut conn = conn.login();

    conn.write(hello.get()).await?;

    Ok(conn)
}
