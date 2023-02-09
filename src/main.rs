mod dns_seed_mananger;
mod constants;
mod network_messages;

use dns_seed_mananger::DnsSeedManager;
use network_messages::new_version_message;

use std::{io::{Write, BufReader}, net::{SocketAddr, Shutdown}};
use std::net::TcpStream;

use bitcoin::{consensus::Decodable};
use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};

#[tokio::main]
async fn main() {    
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


    
    stream.shutdown(Shutdown::Both).unwrap_or_else(|error| {
        eprintln!("Failed to shutdown stream: {:?}", error);
        std::process::exit(1);
    });

}
