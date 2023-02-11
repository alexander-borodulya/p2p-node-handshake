use p2p_node_handshake::DnsSeedManager;
use p2p_node_handshake::HandshakeManager;

#[tokio::main]
async fn main() {
    if let Some(dns_str) = DnsSeedManager::dns_seed_at_index(0) {
        match DnsSeedManager::new_with_dns(dns_str).await {
            Ok(dsm) => {
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
            },
            Err(report) => {
                log::error!("\nError report:\n{:?}", report);
            },
        };
    }
}
