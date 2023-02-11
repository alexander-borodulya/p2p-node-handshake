use std::{
    collections::HashMap,
    io::{BufReader, Write},
    net::{SocketAddr, TcpStream}, fmt, error::Error,
};

use bitcoin::{consensus::Decodable, network::message::RawNetworkMessage};
use tokio::time::timeout;

use error_stack::{
    IntoReport, 
    Report, 
    Result, 
    ResultExt
};

use crate::network_messages;

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

/// Top level handshake error
#[derive(Debug)]
pub struct HandshakeError;

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hhandshake error")
    }
}

impl Error for HandshakeError {}

/// Handshake Timeout Error
#[derive(Debug)]
struct HandshakeTimeoutError;

impl fmt::Display for HandshakeTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handshake error: Main handshake function failed")
    }
}

impl Error for HandshakeTimeoutError {}

/// Handshake Thread Error
#[derive(Debug)]
struct HandshakeThreadError;

impl fmt::Display for HandshakeThreadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handshake thread error: Main handshake function failed to join working thread")
    }
}

impl Error for HandshakeThreadError {}

/// Handshake Message Exchange Error
#[derive(Debug)]
struct HandshakeMessageExchangeError;

impl fmt::Display for HandshakeMessageExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handshake message exchange error: Main handshake function failed to send message")
    }
}

impl Error for HandshakeMessageExchangeError {}


/// HandshakeManager - provides handshake functionality. Tracks the status of a handshake by `remote` SocketAddr.
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
    pub async fn do_handshake(&mut self, remote: SocketAddr) -> Result<(), HandshakeError> {
        // 1. Spawn a new task the performs the message exchange
        let handshake_jh = tokio::spawn(async move { exec_handshake(remote).await });

        // 2. Expect the handshake to be completed in specified timeout
        let timeout_result = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            handshake_jh,
        ).await;

        let jh_result = timeout_result
            .into_report()
            .change_context(HandshakeError)
            .attach_printable_lazy(|| format!("Handshake timed out after {}ms", self.timeout_ms))?;

        // Handle JoinHandle result
        let hs_result = jh_result
           .into_report()
           .change_context(HandshakeThreadError)
           .attach_printable_lazy(|| format!("Handshake thread failed to join"))
           .change_context(HandshakeError)?;

        // Handle Handshake result
        let hs_status = hs_result
           .change_context(HandshakeMessageExchangeError)
           .attach_printable_lazy(|| format!("Handshake message exchange failed"))
           .change_context(HandshakeError)?;

        // 3. Evaluate the handshake status
        match hs_status {
            HandshakeStatus::Completed => Ok(()),
            HandshakeStatus::Timeout => Err(Report::new(HandshakeError)
                .attach_printable(format!("Handshake timed out after {}ms", self.timeout_ms))),
            HandshakeStatus::Failed => Err(Report::new(HandshakeError)
                .attach_printable(format!("Handshake failed"))),
        }
    }
}

async fn exec_handshake(remote: SocketAddr) -> Result<HandshakeStatus, HandshakeMessageExchangeError> {
    match TcpStream::connect(remote) {
        Ok(mut stream) => {
            let read_stream = stream.try_clone()
                .into_report()
                .attach_printable_lazy(|| format!("Failed to clone handshake stream"))
                .change_context(HandshakeMessageExchangeError)?;
            let mut stream_reader = BufReader::new(read_stream);
            let local_peer: SocketAddr = stream.local_addr()
                .into_report()
                .attach_printable_lazy(|| format!("Failed to return local half of the TCP connection"))
                .change_context(HandshakeMessageExchangeError)?;
            let remote_peer: SocketAddr = stream.peer_addr()
               .into_report()
               .attach_printable_lazy(|| format!("Failed to return remote half of the TCP connection"))
               .change_context(HandshakeMessageExchangeError)?;

            // Make and send Version message
            let version_message_bytes = network_messages::new_version_message_serialised(local_peer, remote_peer);
            stream.write_all(version_message_bytes.as_slice())
               .into_report()
               .attach_printable_lazy(|| format!("Failed to send Version message"))
               .change_context(HandshakeMessageExchangeError)?;

            // Wait for the version message from the remote peer
            let message_version_remote = RawNetworkMessage::consensus_decode(&mut stream_reader)
                .into_report()
                .attach_printable_lazy(|| format!("Failed to receive and decode Version message from the remote peer"))
                .change_context(HandshakeMessageExchangeError)?;
            let message_version_remote = message_version_remote.payload;

            println!("RECV: Remote VERSION: {:?}", message_version_remote);

            // Make and send VerAck message to the remote peer
            let message_verack_bytes = network_messages::make_verack_message_serialised();
            stream
                .write_all(message_verack_bytes.as_slice())
                .into_report()
                .attach_printable_lazy(|| format!("Failed to send VerAck message to the remote peer"))
                .change_context(HandshakeMessageExchangeError)?;

            // Wait for the VerAck message from the remote peer
            let message_verack_remote = RawNetworkMessage::consensus_decode(&mut stream_reader)
               .into_report()
               .attach_printable_lazy(|| format!("Failed to receive and decode VerAck message from the remote peer"))
               .change_context(HandshakeMessageExchangeError)?;
            let message_verack_remote = message_verack_remote.payload;

            println!("RECV: Remote Verack: {:?}", message_verack_remote);
        },
        Err(e) => {
            return Err(Report::new(HandshakeMessageExchangeError)
               .attach_printable(format!("Failed to connect to node: {:?}, error: {:?}", remote, e)));
        }
    }

    Ok(HandshakeStatus::Completed)
}
