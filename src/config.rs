use std::fmt;
use std::error::Error;
use error_stack::{IntoReport, Report, Result, ResultExt};
use log::{info, error};

use crate::{DnsSeedManager, HandshakeManager};

const CLI_COMMAND_LIST_DNS_RESOLVERS: &str = "-l";
const CLI_COMMAND_RESOLVE_PEER_URLS: &str = "-r";
const CLI_COMMAND_HANDSHAKE_BY_INDEX: &str = "-hbi";
const CLI_COMMAND_HANDSHAKE_BY_URL: &str = "-hbu";

/// CLI argument parser and command handler
///
/// Supported arguments:
///
/// `-l` - Prints a list of available DNS resolvers.
/// 
///        Example output:
/// 
///             `cargo run -- -l`
///             
///             0 - https://dns-resolver-url-0.com
///             1 - https://dns-resolver-url-1.com
///             2 - https://dns-resolver-url-2.com
///
/// `-r <DNS URL>` - Resolves remote peer URLs by specified DNS URL.
///
/// `-hbi <REMOTE PEER URL>` - Performs a handshake with a specified peer.
/// 
///       `cargo run -- -r {DNS URL}`
///
/// `-hbu <DNS URL INDEX> <REMOTE PEER URL INDEX>` - Performs a handshake 
///       with remote peer by specified URL index.
///       Index corresponds to the URL index in the list of resolved URLs.
///       List of resolved URLs can be obtained by running:
///
///           `cargo run -- -r <DNS URL>`
#[derive(Debug)]
pub struct Config {
    pub command: String,
    pub arguments: Vec<String>,
}

#[derive(Debug)]
pub struct ConfigError;
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config error")
    }
}
impl Error for ConfigError {}


/// ConfigBuild error
#[derive(Debug)]
pub struct ConfigBuildError;

impl fmt::Display for ConfigBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config build error")
    }
}

impl Error for ConfigBuildError {}

/// Config Run Error
#[derive(Debug)]
pub struct ConfigRunError;

impl fmt::Display for ConfigRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config run error")
    }
}

impl Error for ConfigRunError {}


impl Config {
    /// Collects CLI arguments and returns a Config struct
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, ConfigError> {
        // skip the program name
        args.next(); 

        let command = match args.next() {
            Some(arg) => arg,
            None => {
                return Err(Report::new(ConfigBuildError)
                    .attach_printable("Not command specified")
                    .change_context(ConfigError)
                );
            }
        };

        let mut arguments = Vec::new();

        while let Some(arg) = args.next() {
            arguments.push(arg);
        }

        Ok(Config {
            command,
            arguments
        })
    }
}

/// Converts a string representation of a config into a number
fn argument_to_number(args: &Vec<String>, i: usize) -> Result<usize, ConfigError> {
    let Some(dns_index) = args.get(i) else {
        return Err(Report::new(ConfigError)
            .attach_printable(format!("Argument at index 0 is not found")));
    };

    dns_index.parse()
        .into_report()
        .attach_printable_lazy(|| format!("Could not convert String to usize: {}", dns_index))
        .change_context(ConfigError)
}

/// Runs the handshake in accordance with the provided configuration.
/// Returns result that represents the status of the handshake.
pub async fn run(config: &Config) -> Result<(), ConfigError> {
    match config.command.as_str() {
        CLI_COMMAND_LIST_DNS_RESOLVERS => {
            info!("DNS Resolvers:");
            DnsSeedManager::print_default_dns_seeds();
        },
        CLI_COMMAND_RESOLVE_PEER_URLS => {
            info!("Active IP node URLs:");
            let dns_index = argument_to_number(&config.arguments, 0)?;
            let dsm = DnsSeedManager::new_with_dns_index(dns_index).await
            .change_context(ConfigError)?;
            dsm.print_resolved_remote_urls();
        },
        CLI_COMMAND_HANDSHAKE_BY_INDEX => {
            info!("Handshake by DNS seed and IP indexes...");
            
            let dns_url_index = argument_to_number(&config.arguments, 0)?;
            let _dns_url = DnsSeedManager::dns_seed_at_index(dns_url_index).unwrap();
            
            let dsm = DnsSeedManager::new_with_dns_index(dns_url_index)
                .await
                .change_context(ConfigError)?;
                
            let remote_peer_index = argument_to_number(&config.arguments, 1)?;
            let mut handshake_manager = HandshakeManager::default();
            let Some(remote) = dsm.get(remote_peer_index) else {
                return Err(Report::new(ConfigError)
                   .attach_printable(format!("Bad remote peer index: {:?}", remote_peer_index))
                   .change_context(ConfigError));
            };

            let remote = remote.clone();
            match handshake_manager.establish_handshake(remote).await {
                Ok(_s) => {
                    info!("Handshake with IP {:?} evaluated from DNS seed index {:?} and IP index {:?}, completed", remote, dns_url_index, remote_peer_index);
                    handshake_manager.record_handshake(remote, _s);
                }
                Err(e) => {
                    handshake_manager.record_handshake(remote, false);
                    error!("Handshake with IP {:?} evaluated from DNS seed index {:?} and IP index {:?}, failed. Error:\n{:?}", remote, dns_url_index, remote_peer_index, e);
                }
            }
        },
        CLI_COMMAND_HANDSHAKE_BY_URL => {
            info!("Handshake by IP URL...");

            let mut handshake_manager = HandshakeManager::default();

            let Some(sockaddr_string) = config.arguments.get(0) else {
                return Err(Report::new(ConfigError)
                    .attach_printable(format!("Argument at index 0 is not found")));
            };

            let remote = sockaddr_string.parse()
                .into_report()
                .attach_printable_lazy(|| format!("Could not parse IP address: {sockaddr_string:?}"))
                .change_context(ConfigError)?;

            let hs_status = match handshake_manager.establish_handshake(remote).await {
                Ok(established) => {
                    info!("handshake completed successfully with node: {remote}");
                    established
                }
                Err(e) => {
                    eprintln!("Handshake with remote peer {remote:?} failed with error: \n{e:?}");
                    false
                }
            };
            handshake_manager.record_handshake(remote, hs_status);
        },
        _ => {
            return Err(Report::new(ConfigRunError)
                .attach_printable(format!("Invalid command provided: {:?}", config.command)))
                .change_context(ConfigError);
        }
    }
    Ok(())
}
