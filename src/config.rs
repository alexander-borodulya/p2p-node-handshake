use log::info;

use crate::{DnsSeedManager, HandshakeManager};

const CLI_COMMAND_LIST_DNS_RESOLVERS: &str = "-l";
const CLI_COMMAND_RESOLVE_PEER_URLS: &str = "-r";
const CLI_COMMAND_HANDSHAKE_BY_INDEX: &str = "-hs";
const CLI_COMMAND_HANDSHAKE_BY_URL: &str = "-hp";

/// CLI argument parser and command handler
/// 
/// Supported arguments:
/// 
/// `-l` - Prints a list of available DNS resolvers.
///        Example output:
///             > cargo run -- -l
///             
///             0 - https://dns-resolver-url-0.com
///             1 - https://dns-resolver-url-1.com
///             2 - https://dns-resolver-url-2.com
/// 
/// `-r <DNS URL>` - Resolves remote peer URLs by specified DNS URL.
/// 
/// `-hp <REMOTE PEER URL>` - Performs a handshake with a specified peer.
///       > cargo run -- -r {DNS URL}
/// 
/// `-hs <DNS URL INDEX> <REMOTE PEER URL INDEX>` 
/// 
///     - Performs a handshake with remote peer by specified URL index.
///       Index corresponds to the URL index in the list of resolved URLs.
///       List of resolved URLs can be obtained by running: 
/// 
///           > cargo run -- -r <DNS URL>
#[derive(Debug)]
pub struct Config {
    pub command: String,
    pub arguments: Vec<String>,
}

impl Config {
    /// Collects CLI arguments and returns a Config struct
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // skip the program name

        let command = match args.next() {
            Some(arg) => arg,
            None => {
                return Err("No command provided!");
            }
        };
                
        let mut arguments = Vec::new();

        while let Some(arg) = args.next() {
            arguments.push(arg);
        }

        Ok(Config {
            command, arguments
        })
    }

    fn list_dns_resolvers(&self) -> bool {
        self.command.contains(CLI_COMMAND_LIST_DNS_RESOLVERS)
    }

    fn resolve_peer_urls(&self) -> bool {
        self.command.contains(CLI_COMMAND_RESOLVE_PEER_URLS) && self.arguments.len() == 1
    }

    fn handshake_by_url(&self) -> bool {
        self.command.contains(CLI_COMMAND_HANDSHAKE_BY_URL) && self.arguments.len() == 1
    }

    fn handshake_by_index(&self) -> bool {
        self.command.contains(CLI_COMMAND_HANDSHAKE_BY_INDEX) && self.arguments.len() == 2
    }
}

pub async fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.list_dns_resolvers() {
        info!("DNS Resolvers:");
        DnsSeedManager::print_default_dns_seeds();
    }

    if config.resolve_peer_urls() {
        info!("Peer URLs:");
        let dns_index = config.arguments[0].parse().unwrap();
        let dsm = DnsSeedManager::new_with_dns_index(dns_index).await.unwrap();
        dsm.print_resolved_remote_urls();
    }

    if config.handshake_by_url() {
        info!("Handshake with peer: {}", config.arguments[0]);
        
        let mut handshake_manager = HandshakeManager::default();
        let remote = config.arguments[0].parse().unwrap();
        let r = handshake_manager.establish_handshake(remote).await;

        match r {
            Ok(_s) => {
                info!("handshake completed successfully with node: {remote}");
                handshake_manager.record_handshake(remote, _s);
            },
            Err(e) => {
                handshake_manager.record_handshake(remote, false);
                eprintln!("Handshake with remote peer {remote:?} failed with error: {e:?}");
                log::error!("\nError report:\n{:?}", e);
            },
        }
    }
    
    if config.handshake_by_index() {
        
        let dns_url_index = config.arguments[0].parse().unwrap();
        let _dns_url = DnsSeedManager::dns_seed_at_index(dns_url_index).unwrap();
        let dsm = DnsSeedManager::new_with_dns_index(dns_url_index).await.unwrap();
        
        let remote_peer_index = config.arguments[1].parse().unwrap();
        let mut handshake_manager = HandshakeManager::default();
        let remote = dsm.get(remote_peer_index).unwrap().clone();
        
        info!("Handshake with peer by indexes: {:?}, DNS: {:?}, REMOTE: {:?}", config.arguments, _dns_url, remote);
        let r = handshake_manager.establish_handshake(remote).await;
        match r {
            Ok(_s) => {
                info!("handshake completed successfully with node: {remote}");
                handshake_manager.record_handshake(remote, _s);
            },
            Err(e) => {
                handshake_manager.record_handshake(remote, false);
                eprintln!("Handshake with remote peer {remote:?} failed with error: {e:?}");
                log::error!("\nError report:\n{:?}", e);
            },
        }
    }

    Ok(())
}
