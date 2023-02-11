mod constants;
mod dns_seed_mananger;
mod handshake_manager;
mod network_messages;

use crate::dns_seed_mananger::DnsSeedManager;
use crate::handshake_manager::HandshakeManager;

#[tokio::main]
async fn main() {
    let dsm = DnsSeedManager::default();
    let mut handshake_manager = HandshakeManager::default();

    for remote in dsm.seeds {
        let r = handshake_manager.do_handshake(remote).await;

        match r {
            Ok(()) => {
                println!("Handshake with remote peer established: {:?}", remote);
                break;
            }
            Err(e) => {
                println!(
                    "Handshake with remote peer {:?} failed with error: {:?}",
                    remote, e
                );
            }
        }
    }
}
