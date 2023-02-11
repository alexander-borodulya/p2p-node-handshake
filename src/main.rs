use p2p_node_handshake::DnsSeedManager;
use p2p_node_handshake::HandshakeManager;

#[tokio::main]
async fn main() {
    env_logger::init();

    let dsm = DnsSeedManager::default();
    let mut handshake_manager = HandshakeManager::default();

    for remote in dsm.seeds {
        let r = handshake_manager.do_handshake(remote).await;

        match r {
            Ok(()) => {
                println!("Handshake with remote peer established: {:?}", remote);
                break;
            },
            Err(e) => {
                println!("Handshake with remote peer {remote:?} failed with error: {e:?}");
                log::error!("\nError report:\n{:?}", e);
            },
        }
    }
}
