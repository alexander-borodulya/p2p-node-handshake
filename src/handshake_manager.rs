use bitcoin::{
    consensus::Decodable,
    network::message::{NetworkMessage, RawNetworkMessage},
};
use error_stack::{IntoReport, Report, Result, ResultExt};
use log::{error, info};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    io::{BufReader, Write},
    net::{SocketAddr, TcpStream},
};
use tokio::time::timeout;

use crate::network_messages;

/// Top level handshake error - i.e. general error
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
        write!(
            f,
            "Handshake thread error: Main handshake function failed to join working thread"
        )
    }
}

impl Error for HandshakeThreadError {}

/// Handshake Message Exchange Error
#[derive(Debug)]
struct HandshakeMessageExchangeError;

impl fmt::Display for HandshakeMessageExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Handshake message exchange error: Main handshake function failed to send message"
        )
    }
}

impl Error for HandshakeMessageExchangeError {}

/// Handshake Message Wrong Protocol Error
#[derive(Debug)]
struct HandshakeMessageWrongProtocolError;

impl fmt::Display for HandshakeMessageWrongProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handshake message wrong protocol error: Main handshake function failed to send message")
    }
}

impl Error for HandshakeMessageWrongProtocolError {}

/// Handshake Message VerAck Error
#[derive(Debug)]
struct HandshakeMessageVerAckError;

impl fmt::Display for HandshakeMessageVerAckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Handshake message verack error: Main handshake function failed to send message"
        )
    }
}

impl Error for HandshakeMessageVerAckError {}

/// HandshakeManager - provides handshake functionality. Tracks the status of a handshake by `remote` SocketAddr.
pub struct HandshakeManager {
    timeout_ms: u64,
    statuses: HashMap<SocketAddr, bool>,
}

/// Default trait implementation for `HandshakeManager`
impl Default for HandshakeManager {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            statuses: HashMap::new(),
        }
    }
}

impl HandshakeManager {
    /// Perform a handshake with a `remote` SocketAddr.
    /// Returns `true` if the handshake was successful, `false` otherwise.
    pub async fn establish_handshake(
        &mut self,
        remote: SocketAddr,
    ) -> Result<bool, HandshakeError> {
        // 1. Spawn a new task the performs the message exchange
        let handshake_jh = tokio::spawn(async move { exec_handshake(remote).await });

        // 2. Expect the handshake to be completed in specified timeout
        let timeout_result = timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            handshake_jh,
        )
        .await;

        // Handle Timeout result
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

        Ok(hs_status)
    }

    /// Adde record entry to the handshake statuses
    pub fn record_handshake(&mut self, remote: SocketAddr, status: bool) {
        self.statuses.insert(remote, status);
    }

    /// Print all recorded handshake statuses into the terminal
    pub fn _print_statuses(&self) {
        for (addr, status) in self.statuses.iter() {
            info!("Remote peer: {}, handshake status: {}", addr, status);
        }
    }
}

/// Implements version handshake protocol as follows:
///
/// =============================================================================
///
///     L -> R: Send version message with the local peer's version
///     R -> L: Send version message back
///     R -> L: Send verack message
///     R:      Sets version to the minimum of the 2 versions
///     L -> R: Send verack message after receiving version message from R
///     L:      Sets version to the minimum of the 2 versions
///
/// =============================================================================
///
/// Returns result that indicates if the handshake was successful or not.
/// Failed message exchange error represented by `HandshakeMessageExchangeError`.
async fn exec_handshake(remote: SocketAddr) -> Result<bool, HandshakeMessageExchangeError> {
    match TcpStream::connect(remote) {
        Ok(mut stream) => {
            let read_stream = stream
                .try_clone()
                .into_report()
                .attach_printable_lazy(|| format!("Failed to clone handshake stream"))
                .change_context(HandshakeMessageExchangeError)?;
            let mut stream_reader = BufReader::new(read_stream);
            let local_peer: SocketAddr = stream
                .local_addr()
                .into_report()
                .attach_printable_lazy(|| {
                    format!("Failed to return local half of the TCP connection")
                })
                .change_context(HandshakeMessageExchangeError)?;
            let remote_peer: SocketAddr = stream
                .peer_addr()
                .into_report()
                .attach_printable_lazy(|| {
                    format!("Failed to return remote half of the TCP connection")
                })
                .change_context(HandshakeMessageExchangeError)?;

            // Make and send Version message
            let (protocol_version_local, version_message_bytes) =
                network_messages::new_version_message_serialised(local_peer, remote_peer);
            info!("Send version message {protocol_version_local} to {remote}");
            stream
                .write_all(version_message_bytes.as_slice())
                .into_report()
                .attach_printable_lazy(|| format!("Failed to send Version message"))
                .change_context(HandshakeMessageExchangeError)?;

            // Wait for the version message from the remote peer
            let message_version_remote = RawNetworkMessage::consensus_decode(&mut stream_reader)
                .into_report()
                .attach_printable_lazy(|| {
                    format!("Failed to receive and decode Version message from the remote peer")
                })
                .change_context(HandshakeMessageExchangeError)?;
            let message_version_remote = message_version_remote.payload;

            let protocol_version_remote = match message_version_remote {
                NetworkMessage::Version(protocol_version_remote) => protocol_version_remote.version,
                _ => {
                    return Err(
                        Report::new(HandshakeMessageWrongProtocolError).attach_printable(format!(
                            "Received unexpected protocol version: {:?}",
                            message_version_remote
                        )),
                    )
                    .change_context(HandshakeMessageExchangeError)
                }
            };
            info!("Recv version message {protocol_version_remote} from {remote}");

            // Make and send VerAck message to the remote peer
            let message_verack_bytes = network_messages::make_verack_message_serialised();
            stream
                .write_all(message_verack_bytes.as_slice())
                .into_report()
                .attach_printable_lazy(|| {
                    format!("Failed to send VerAck message to the remote peer")
                })
                .change_context(HandshakeMessageExchangeError)?;
            info!("Sent VerAck message to {remote}");

            // Wait for the VerAck message from the remote peer
            let message_verack_remote = RawNetworkMessage::consensus_decode(&mut stream_reader)
                .into_report()
                .attach_printable_lazy(|| {
                    format!("Failed to receive and decode VerAck message from the remote peer")
                })
                .change_context(HandshakeMessageExchangeError)?;

            let message_verack_remote = match message_verack_remote.payload {
                NetworkMessage::Verack => message_verack_remote.payload,
                _ => {
                    error!(
                        "Received unexpected message, but expected VerAck message: {:?}",
                        message_verack_remote.payload
                    );
                    return Err(
                        Report::new(HandshakeMessageVerAckError).attach_printable(format!(
                            "Received unexpected message, but expected VerAck message: {:?}",
                            message_verack_remote.payload
                        )),
                    )
                    .change_context(HandshakeMessageExchangeError);
                }
            };

            info!("Recv VerAck message from {remote}: {message_verack_remote:?}");
        }
        Err(e) => {
            return Err(
                Report::new(HandshakeMessageExchangeError).attach_printable(format!(
                    "Failed to connect to node: {:?}, error: {:?}",
                    remote, e
                )),
            );
        }
    }
    Ok(true)
}
