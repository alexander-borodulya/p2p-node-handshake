use bitcoin::network::{
    constants::ServiceFlags,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
    Address,
};
use rand::Rng;
use std::net;

use crate::constants;

/// Builds and returns a version message tuple
pub fn new_version_message(
    local_peer: net::SocketAddr,
    remote_peer: net::SocketAddr,
) -> (u32, NetworkMessage) {
    const SERVICES: ServiceFlags = ServiceFlags::NONE;

    let timestamp = chrono::Utc::now().timestamp();
    let receiver = Address::new(&remote_peer, SERVICES);
    let sender = Address::new(&local_peer, SERVICES);
    let nonce = rand::thread_rng().gen();
    let user_agent = "user-agent-bitcoin-p2p-handshake".to_owned();
    let start_height = 0;

    // Construct the message
    let mut message = VersionMessage::new(
        SERVICES,
        timestamp,
        receiver,
        sender,
        nonce,
        user_agent,
        start_height,
    );

    message.version = constants::PROTOCOL_VERSION;

    (message.version, NetworkMessage::Version(message))
}

/// Make RawVersion message and serealize it. Returns a tuple of (protocol_verion, serealized_message)
pub fn new_version_message_serialised(
    local_peer: net::SocketAddr,
    remote_peer: net::SocketAddr,
) -> (u32, Vec<u8>) {
    let version_message_tup = new_version_message(local_peer, remote_peer);
    let version_message_local_raw = RawNetworkMessage {
        magic: bitcoin::Network::Bitcoin.magic(),
        payload: version_message_tup.1,
    };
    (
        version_message_tup.0,
        bitcoin::consensus::encode::serialize(&version_message_local_raw),
    )
}

/// Make local VerAck message and serealize it into bytes.
pub fn make_verack_message_serialised() -> Vec<u8> {
    let message_verack_local = NetworkMessage::Verack;
    let message_verack_local_raw = RawNetworkMessage {
        magic: bitcoin::Network::Bitcoin.magic(),
        payload: message_verack_local,
    };
    bitcoin::consensus::encode::serialize(&message_verack_local_raw)
}
