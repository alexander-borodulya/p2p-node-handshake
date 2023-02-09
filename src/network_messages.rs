use std::net;
use bitcoin::network::{message::NetworkMessage, constants::ServiceFlags, Address, message_network::VersionMessage};
use rand::Rng;

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
    let message = VersionMessage::new(
        SERVICES,
        timestamp,
        receiver,
        sender,
        nonce,
        user_agent,
        start_height,
    );

    NetworkMessage::Version(message)
}
