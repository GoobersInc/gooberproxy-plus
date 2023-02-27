use azalea_auth::{game_profile::GameProfile, sessionserver::ClientSessionServerError};
use azalea_chat::FormattedText;
use azalea_client::Account;
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
    #[error("disconnected: {0}")]
    Disconnected(FormattedText),

    #[error("invalid account: account has no access token or UUID")]
    InvalidAccount,

    #[error("unexpected packet: {0:?}")]
    UnexpectedPacket(ClientboundLoginPacket),

    #[error(transparent)]
    Connection(#[from] ConnectionError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ReadPacket(#[from] Box<ReadPacketError>),

    #[error(transparent)]
    SessionServer(#[from] ClientSessionServerError),
}

/// Start a connection, authenticate and join the server
/// without sending the encryption response
///
/// This function assumes that `account` has an access token and UUID
pub async fn almost_join_server(
    addr: &SocketAddr,
    account: &Account,
) -> Result<(ClientLoginConn, ServerboundKeyPacket, [u8; 16]), JoinServerError> {
    if account.access_token.is_none() || account.uuid.is_none() {
        return Err(JoinServerError::InvalidAccount);
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

    // Hello!
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

    // Hello?
    let encryption_request = match conn.read().await? {
        ClientboundLoginPacket::Hello(packet) => packet,
        ClientboundLoginPacket::LoginDisconnect(packet) => {
            return Err(JoinServerError::Disconnected(packet.reason));
        }
        packet => {
            return Err(JoinServerError::UnexpectedPacket(packet));
        }
    };

    // Lettuce do le auth (auth so goofy)
    // ↑ this file has been copied so much from project to project
    // that I don't even know who wrote this anymore
    let encryption_result =
        azalea_crypto::encrypt(&encryption_request.public_key, &encryption_request.nonce).unwrap();

    // Very secur (I hope)
    let secret_key = encryption_result.secret_key.to_owned();

    // Here we actually do the auth smh
    let access_token = { account.access_token.as_ref().unwrap().lock().to_owned() };
    conn.authenticate(
        &access_token,
        &account.uuid.unwrap(),
        secret_key,
        &encryption_request,
    )
    .await?;

    Ok((
        conn,
        ServerboundKeyPacket {
            key_bytes: encryption_result.encrypted_public_key,
            nonce_or_salt_signature: NonceOrSaltSignature::Nonce(encryption_result.encrypted_nonce),
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

    // While this could technically be abused to cause the client to
    // loop forever, it would have to be a very targeted attack
    loop {
        match conn.read().await? {
            ClientboundLoginPacket::GameProfile(packet) => {
                return Ok((conn.game(), packet.game_profile));
            }
            ClientboundLoginPacket::LoginCompression(packet) => {
                conn.set_compression_threshold(packet.compression_threshold);
            }
            ClientboundLoginPacket::LoginDisconnect(packet) => {
                return Err(JoinServerError::Disconnected(packet.reason));
            }
            packet => {
                return Err(JoinServerError::UnexpectedPacket(packet));
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

/// Send a handshake and switch to login
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
