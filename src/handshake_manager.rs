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
