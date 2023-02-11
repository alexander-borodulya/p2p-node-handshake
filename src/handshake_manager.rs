use std::{
    collections::HashMap,
    io::{BufReader, Write},
    net::{SocketAddr, TcpStream},
};

use bitcoin::{consensus::Decodable, network::message::RawNetworkMessage};
use tokio::time::timeout;

use crate::network_messages::{make_verack_message_serialised, new_version_message_serialised};

#[derive(Debug)]
pub enum HandshakeStatus {
    /// Remote peer correctly responded to the handshake.
    Completed,

    /// Remote peer did not respond to the handshake in specified timeout.
    Timeout,

    /// Error occurred during handshake.
    Failed,
}

pub type HandshakeResult = Result<(), std::io::Error>;

pub struct HandshakeManager {
    timeout_ms: u64,
    statuses: HashMap<SocketAddr, HandshakeResult>,
}

impl Default for HandshakeManager {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            statuses: HashMap::new(),
        }
    }
}

impl HandshakeManager {
    pub async fn do_handshake(&mut self, remote: SocketAddr) -> HandshakeResult {
        // 1. Spawn a new task the performs the message exchange
        let handshake_jh = tokio::spawn(async move { exec_handshake(remote).await });

        // 2. Expect the handshake to be completed in specified timeout
        let timeout_result = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            handshake_jh,
        )
        .await;

        // Handle timeout result
        let jh_result = match timeout_result {
            Ok(jh_result) => jh_result,
            Err(error) => {
                eprintln!("Handshake timeout reached: {:?}", error);
                return Err(std::io::Error::from(std::io::ErrorKind::TimedOut));
            }
        };

        // Handle JoinHandle result
        let hs_result = match jh_result {
            Ok(hs_result) => hs_result,
            Err(e) => {
                eprintln!("Handshake joinhandle error: {:?}", e);
                return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
            }
        };

        // Handle Handshake result
        let hs_status = match hs_result {
            Ok(hs_status) => hs_status,
            Err(e) => {
                eprintln!("Handshake error: {:?}", e);
                return Err(std::io::Error::from(std::io::ErrorKind::ConnectionRefused));
            }
        };

        match hs_status {
            HandshakeStatus::Completed => Ok(()),
            HandshakeStatus::Timeout => Err(std::io::Error::from(std::io::ErrorKind::TimedOut)),
            HandshakeStatus::Failed => Err(std::io::Error::from(std::io::ErrorKind::Other)),
        }
    }
}

async fn exec_handshake(remote: SocketAddr) -> Result<HandshakeStatus, std::io::Error> {
    match TcpStream::connect(remote) {
        Ok(mut stream) => {
            let read_stream = stream.try_clone()?;
            let mut stream_reader = BufReader::new(read_stream);
            let local_peer: SocketAddr = stream.local_addr()?;
            let remote_peer: SocketAddr = stream.peer_addr()?;

            // Make and send Version message
            let version_message_bytes = new_version_message_serialised(local_peer, remote_peer);
            stream.write_all(version_message_bytes.as_slice())?;

            // Wait for the version message from the remote peer
            let message_version_remote = RawNetworkMessage::consensus_decode(&mut stream_reader)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "Failed to receive Version message form the remote peer: {:?}",
                        error
                    );
                    std::process::exit(1);
                });
            let message_version_remote = message_version_remote.payload;
            println!("RECV: Remote VERSION: {:?}", message_version_remote);

            // Make and send VerAck message to the remote peer
            let message_verack_bytes = make_verack_message_serialised();
            stream
                .write_all(message_verack_bytes.as_slice())
                .unwrap_or_else(|error| {
                    eprintln!("Failed to send VerAck message: {:?}", error);
                    std::process::exit(1);
                });

            // Wait for the VerAck message from the remote peer
            let message_verack_remote = RawNetworkMessage::consensus_decode(&mut stream_reader)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "Failed to receive VerAck message form the remote peer: {:?}",
                        error
                    );
                    std::process::exit(1);
                });
            let message_verack_remote = message_verack_remote.payload;

            println!("RECV: Remote Verack: {:?}", message_verack_remote);
        }
        Err(e) => {
            eprintln!("Failed to connect to node: {:?}, error: {:?}", remote, e);
            return Err(e);
        }
    }

    Ok(HandshakeStatus::Completed)
}
