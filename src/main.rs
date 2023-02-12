use log::info;
use p2p_node_handshake::DnsSeedManager;
use p2p_node_handshake::HandshakeManager;

#[tokio::main]
async fn main() {
    let env = env_logger::Env::default().filter_or("log-level-info", "info");
    env_logger::init_from_env(env);

    let dsm = DnsSeedManager::default();
    let mut handshake_manager = HandshakeManager::default();

    for remote in dsm.seeds {
        info!("About to handshake with {}", remote);
        let r = handshake_manager.establish_handshake(remote).await;
        match r {
            Ok(_s) => {
                info!("handshake completed successfully with node: {remote}");
                handshake_manager.record_handshake(remote, _s);
                break;
            },
            Err(e) => {
                handshake_manager.record_handshake(remote, false);
                eprintln!("Handshake with remote peer {remote:?} failed with error: {e:?}");
                log::error!("\nError report:\n{:?}", e);
            },
        }
    }

    handshake_manager.print_statuses();
    
}
