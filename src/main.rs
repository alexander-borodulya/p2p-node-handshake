mod dns_seed_mananger;
mod handshake_manager;
mod constants;
mod network_messages;

use dns_seed_mananger::DnsSeedManager;
use handshake_manager::{HandshakeStatus, HandshakeResult};
use network_messages::new_version_message;
use tokio::{time::{timeout}};

use std::{io::{Write, BufReader, Read}, net::{SocketAddr, Shutdown}, time::Duration, collections::HashMap};
use std::net::TcpStream;

use bitcoin::{consensus::Decodable};
use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};

async fn _exec_handshake(remote: SocketAddr) -> Result<HandshakeStatus, std::io::Error> {
    println!("exec_handshake - start");

    let mut stream = TcpStream::connect(remote)?;
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    println!("buffer: {:?}", String::from_utf8(buffer.to_vec()).unwrap());
    stream.shutdown(Shutdown::Both)?;

    println!("exec_handshake - end");
    Ok(HandshakeStatus::Completed)
}

async fn exec_handshake(remote: SocketAddr) -> Result<HandshakeStatus, std::io::Error> {

    match TcpStream::connect(remote) {
        Ok(mut stream) => {

            let read_stream = stream.try_clone()?;
        
            let mut stream_reader = BufReader::new(read_stream);
        
            let local_peer: SocketAddr = stream.local_addr()?;
        
            let remote_peer: SocketAddr = stream.peer_addr()?;

            // Make Version message
            let version_message_local = new_version_message(local_peer, remote_peer);
            // Make RawVersion message and serealize it
            let version_message_local_raw = RawNetworkMessage {
                magic: bitcoin::Network::Bitcoin.magic(),
                payload: version_message_local,
            };
            let version_message_bytes = bitcoin::consensus::encode::serialize(&version_message_local_raw);
            // let version_message_bytes = new_version_message_serialised(local_peer, remote_peer);

            // Send the Version message
            stream.write_all(version_message_bytes.as_slice())?;
            
            

            
            // Wait for the version message from the remote peer
            let message_version_remote = RawNetworkMessage::consensus_decode(&mut stream_reader).unwrap_or_else(|error| {
                eprintln!("Failed to receive Version message form the remote peer: {:?}", error);
                std::process::exit(1);
            });
            let message_version_remote = message_version_remote.payload;
            println!("RECV: Remote VERSION: {:?}", message_version_remote);



            // Send VerAck message to the remote peer
            let message_verack_local = NetworkMessage::Verack;
            let message_verack_local_raw = RawNetworkMessage {
                magic: bitcoin::Network::Bitcoin.magic(),
                payload: message_verack_local,
            };
            let message_verack_local_raw_vec = bitcoin::consensus::encode::serialize(&message_verack_local_raw);

            stream.write_all(message_verack_local_raw_vec.as_slice()).unwrap_or_else(|error| {
                eprintln!("Failed to send VerAck message: {:?}", error);
                std::process::exit(1);
            });



            // Wait for the VerAck message from the remote peer
            let message_verack_remote = RawNetworkMessage::consensus_decode(&mut stream_reader).unwrap_or_else(|error| {
                eprintln!("Failed to receive VerAck message form the remote peer: {:?}", error);
                std::process::exit(1);
            });
            let message_verack_remote = message_verack_remote.payload;

            println!("RECV: Remote Verack: {:?}", message_verack_remote);

        },
        Err(e) => {
            eprintln!("Failed to connect to node: {:?}, error: {:?}", remote, e);
            return Err(e);
        }
    }

    Ok(HandshakeStatus::Completed)
}



#[tokio::main]
async fn main() {
    refac_impl().await;
    // plain_impl().await;
}



async fn refac_impl() {

    // Handshake Manager
    struct HandshakeManager {
        timeout_ms: u64,
        statuses: HashMap<SocketAddr, HandshakeResult>
    }

    impl HandshakeManager {
        pub fn new() -> Self {
            Self {
                timeout_ms: 2000,
                statuses: HashMap::new(),
            }
        }

        pub async fn do_handshake(&mut self, remote: SocketAddr) -> HandshakeResult {

            // 1. Spawn a new task the performs the message exchange
            let handshake_jh = tokio::spawn(async move {
                exec_handshake(remote).await
            });
            
            // 2. Expect the handshake to be completed in specified timeout
            let timeout_result = timeout(
                Duration::from_millis(self.timeout_ms), handshake_jh
            ).await;

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
                },
            };

            match hs_status {
                HandshakeStatus::Completed => Ok(()),
                HandshakeStatus::Timeout => Err(std::io::Error::from(std::io::ErrorKind::TimedOut)),
                HandshakeStatus::Failed => Err(std::io::Error::from(std::io::ErrorKind::Other)),
            }
        }
    }

    {
        let dsm = DnsSeedManager::default();
        let mut handshake_manager = HandshakeManager::new();

        for remote in dsm.seeds {

            let r = handshake_manager.do_handshake(remote).await;

            match r {
                Ok(()) => {
                    println!("Handshake with remote peer established: {:?}", remote);
                    break;
                },
                Err(e) => {
                    println!("Handshake with remote peer {:?} failed with error: {:?}", remote, e);
                }   
            }
        }
    }
}



//
// Initial plain implementation for the reference
//



async fn _plain_impl() {
    let dsm = DnsSeedManager::default();
    let node_addr = dsm.get(0).expect("Unable to get node address by index 0");
    println!("Try handshake with node: {:?}", node_addr);



    let mut stream = TcpStream::connect(node_addr).unwrap_or_else(|error| {
        eprintln!("Failed to connect to node: {:?}, error: {:?}", node_addr, error);
        std::process::exit(1);
    });

    let read_stream = stream.try_clone().unwrap_or_else(|error| {
        eprintln!("Failed to clone TcpStream, error: {:?}", error);
        std::process::exit(1);
    });

    let mut stream_reader = BufReader::new(read_stream);

    let local_peer: SocketAddr = stream.local_addr().unwrap_or_else(|error| {
        eprintln!("Error parsing local_peer address: {:?}", error);
        std::process::exit(1);
    });

    let remote_peer: SocketAddr = stream.peer_addr().unwrap_or_else(|error| {
        eprintln!("Error parsing remote_peer address: {:?}", error);
        std::process::exit(1);
    });
    
    
    
    // Make Version message
    let version_message_local = new_version_message(local_peer, remote_peer);

    // Make RawVersion message and serealize it
    let version_message_local_raw = RawNetworkMessage {
        magic: bitcoin::Network::Bitcoin.magic(),
        payload: version_message_local,
    };
    let version_message_local_raw_vec = bitcoin::consensus::encode::serialize(&version_message_local_raw);

    // Send the Version message
    stream.write_all(version_message_local_raw_vec.as_slice()).unwrap_or_else(|error| {
        eprintln!("Failed to send Version message: {:?}", error);
        std::process::exit(1);
    });
    
    
    
    // Wait for the version message from the remote peer
    let message_version_remote = RawNetworkMessage::consensus_decode(&mut stream_reader).unwrap_or_else(|error| {
        eprintln!("Failed to receive Version message form the remote peer: {:?}", error);
        std::process::exit(1);
    });
    
    let message_version_remote = message_version_remote.payload;
    println!("RECV: Remote VERSION: {:?}", message_version_remote);



    // Send VerAck message to the remote peer
    let message_verack_local = NetworkMessage::Verack;
    let message_verack_local_raw = RawNetworkMessage {
        magic: bitcoin::Network::Bitcoin.magic(),
        payload: message_verack_local.clone(),
    };
    let message_verack_local_raw_vec = bitcoin::consensus::encode::serialize(&message_verack_local_raw);

    stream.write_all(message_verack_local_raw_vec.as_slice()).unwrap_or_else(|error| {
        eprintln!("Failed to send VerAck message: {:?}", error);
        std::process::exit(1);
    });



    // Wait for the VerAck message from the remote peer
    let message_verack_remote = RawNetworkMessage::consensus_decode(&mut stream_reader).unwrap_or_else(|error| {
        eprintln!("Failed to receive VerAck message form the remote peer: {:?}", error);
        std::process::exit(1);
    });
    let message_verack_remote = message_verack_remote.payload;

    println!("RECV: Remote Verack: {:?}", message_verack_remote);
    
    
    
    // Check if the local VerAck message is equal to the remote one
    if message_verack_local == message_verack_remote {
        println!("Handshake established with the remote peer: {:?}", remote_peer);
    } else {
        println!("Handshake failed with the remote peer: {:?}", remote_peer);
    }



    stream.shutdown(Shutdown::Both).unwrap_or_else(|error| {
        eprintln!("Failed to shutdown stream: {:?}", error);
        std::process::exit(1);
    });
}
