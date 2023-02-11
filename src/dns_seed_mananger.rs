/// DNS Seeds
///
/// Predefined DNS seed taken from:
///     https://github.com/bitcoin/bitcoin/blob/v24.0.1/src/chainparams.cpp#L123
///
/// "seed.bitcoin.sipa.be."          
/// "dnsseed.bluematt.me."           
/// "dnsseed.bitcoin.dashjr.org."    
/// "seed.bitcoinstats.com."         
/// "seed.bitcoin.jonasschnelli.ch." 
/// "seed.btc.petertodd.org."        
/// "seed.bitcoin.sprovoost.nl."     
/// "dnsseed.emzy.de."               
/// "seed.bitcoin.wiz.biz."          
///
use std::net;

use error_stack::{IntoReport, Result, ResultExt};

type VecSocketAddr = Vec<std::net::SocketAddr>;

const DEFAULT_PORT_MAINNET: u16 = 8333;
const DEFAULT_DNS_SEEDS: &'static [&'static str] = &[
    "seed.bitcoin.sipa.be.",
    "dnsseed.bluematt.me.",
    "dnsseed.bitcoin.dashjr.org.",
    "seed.bitcoinstats.com.",
    "seed.bitcoin.jonasschnelli.ch.",
    "seed.btc.petertodd.org.",
    "seed.bitcoin.sprovoost.nl.",
    "dnsseed.emzy.de.",
    "seed.bitcoin.wiz.biz.",
];

/// DnsLookupError used to indicate an error with the DNS lookup.
#[derive(Debug)]
pub struct DnsLookupError;

impl std::fmt::Display for DnsLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DNS seed lookup failed")
    }
}

impl std::error::Error for DnsLookupError {}

/// DnsSeedManager contains a list of resolved IP addresses
#[derive(Debug)]
pub struct DnsSeedManager {
    pub seeds: Vec<std::net::SocketAddr>,
}

impl Default for DnsSeedManager {
    fn default() -> Self {
        Self {
            seeds: DnsSeedManager::lookup_dns_seeds(
                &DEFAULT_DNS_SEEDS, 
                DEFAULT_PORT_MAINNET
            ),
        }
    }
}

impl DnsSeedManager {
    pub fn new() -> Self {
        Self { seeds: Vec::new() }
    }

    pub async fn new_with_dns(dns: &str) -> Result<Self, DnsLookupError> {
        let mut dsm = DnsSeedManager::new();
        let dns_seed_addr = (dns, DEFAULT_PORT_MAINNET);

        let seeds = tokio::net::lookup_host(dns_seed_addr).await
            .into_report()
            .attach_printable_lazy(|| format!("Failed to lookup dns seeds by URL {:?}", dns_seed_addr))
            .change_context(DnsLookupError)?;

        dsm.seeds.extend(seeds.collect::<Vec<std::net::SocketAddr>>());
        Ok(dsm)
    }

    pub fn dns_seeds_count() -> usize {
        DEFAULT_DNS_SEEDS.len()
    }

    pub fn dns_seed_at_index(i: usize) -> Option<&'static &'static str> {
        let o = DEFAULT_DNS_SEEDS.get(i);
        o
    }

    pub fn ip_count(&self) -> usize {
        self.seeds.len()
    }

    pub fn get(&self, i: usize) -> Option<&net::SocketAddr> {
        self.seeds.get(i)
    }

    fn lookup_dns_seeds(dns: &[&str], port: u16) -> VecSocketAddr {
        let mut v: Vec<std::net::SocketAddr> = Vec::new();
        for d in dns.iter() {
            let t = (*d, port);
            let sa = net::ToSocketAddrs::to_socket_addrs(&t);
            if sa.is_ok() {
                v.extend(sa.unwrap());
            }
        }
        v
    }
}
