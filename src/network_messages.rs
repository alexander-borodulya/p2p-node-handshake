use std::net;
use bitcoin::network::{message::{NetworkMessage, RawNetworkMessage}, constants::ServiceFlags, Address, message_network::VersionMessage};
use rand::Rng;

use crate::constants;

/// Builds and returns a version message
pub fn new_version_message(local_peer: net::SocketAddr, remote_peer: net::SocketAddr) -> NetworkMessage {
    const SERVICES: ServiceFlags = ServiceFlags::NONE;

    let timestamp = chrono::Utc::now().timestamp();
    let receiver = Address::new(&remote_peer, SERVICES);
    let sender = Address::new(&local_peer, SERVICES);
    let nonce = rand::thread_rng().gen();
    let user_agent = "bitcoin-p2p-handshake".to_owned();
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

    NetworkMessage::Version(message)
}

/// Make RawVersion message and serealize it
pub fn new_version_message_serialised(local_peer: net::SocketAddr, remote_peer: net::SocketAddr) -> Vec<u8> {
    let version_message_local_raw = RawNetworkMessage {
        magic: bitcoin::Network::Bitcoin.magic(),
        payload: new_version_message(local_peer, remote_peer),
    };   
    bitcoin::consensus::encode::serialize(&version_message_local_raw)
}
