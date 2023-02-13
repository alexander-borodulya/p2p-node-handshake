/// DNS Seeds
///
/// Predefined DNS seed taken from:
///     https://github.com/bitcoin/bitcoin/blob/v24.0.1/src/chainparams.cpp#L123
///
///     "seed.bitcoin.sipa.be."          
///     "dnsseed.bluematt.me."           
///     "dnsseed.bitcoin.dashjr.org."    
///     "seed.bitcoinstats.com."         
///     "seed.bitcoin.jonasschnelli.ch."
///     "seed.btc.petertodd.org."        
///     "seed.bitcoin.sprovoost.nl."     
///     "dnsseed.emzy.de."               
///     "seed.bitcoin.wiz.biz."          
use std::net;

use error_stack::{IntoReport, Report, Result, ResultExt};

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

/// DnsSeedManager contains a list of resolved IP addresses of active nodes
#[derive(Debug)]
pub struct DnsSeedManager {
    pub active_nodes: Vec<std::net::SocketAddr>,
}

impl Default for DnsSeedManager {
    fn default() -> Self {
        Self {
            active_nodes: DnsSeedManager::lookup_active_nodes(
                &DEFAULT_DNS_SEEDS,
                DEFAULT_PORT_MAINNET,
            ),
        }
    }
}

impl DnsSeedManager {
    /// Construct a new DnsSeedManager
    pub fn new() -> Self {
        Self {
            active_nodes: Vec::new(),
        }
    }

    /// Construct a new DnsSeedManager based on index of DNS seed URL
    pub async fn new_with_dns_index(i: usize) -> Result<Self, DnsLookupError> {
        let Some(dns_url) = DnsSeedManager::dns_seed_at_index(i) else {
            return Err(Report::from(DnsLookupError).attach_printable(format!("Bad DNS seed index: {}", i)));
        };
        DnsSeedManager::new_with_dns(&dns_url).await
    }

    /// Construct a new DnsSeedManager based on DNS seed URL represented as `&str`
    pub async fn new_with_dns(dns: &str) -> Result<Self, DnsLookupError> {
        let mut dsm = DnsSeedManager::new();
        let dns_seed_addr = (dns, DEFAULT_PORT_MAINNET);

        let seeds = tokio::net::lookup_host(dns_seed_addr)
            .await
            .into_report()
            .attach_printable_lazy(|| {
                format!("Failed to lookup dns seeds by URL {:?}", dns_seed_addr)
            })
            .change_context(DnsLookupError)?;

        dsm.active_nodes
            .extend(seeds.collect::<Vec<std::net::SocketAddr>>());
        Ok(dsm)
    }

    /// Return the list of internal DNS seed URLs
    pub fn default_dns_seeds() -> &'static [&'static str] {
        DEFAULT_DNS_SEEDS
    }

    /// Prints the list of internal DNS seed URLs
    pub fn print_default_dns_seeds() {
        for (i, s) in DnsSeedManager::default_dns_seeds().iter().enumerate() {
            println!("{}: {}", i, s);
        }
    }

    /// Prints the list of IP addresses of active nodes
    pub fn print_resolved_remote_urls(&self) {
        for (i, s) in self.active_nodes.iter().enumerate() {
            println!("{}: {}", i, s);
        }
    }

    /// Return DNS seed URL by given index
    pub fn dns_seed_at_index(i: usize) -> Option<&'static &'static str> {
        let o = DEFAULT_DNS_SEEDS.get(i);
        o
    }

    /// Returns IP address of active node by given index
    pub fn get(&self, i: usize) -> Option<&net::SocketAddr> {
        self.active_nodes.get(i)
    }

    /// Accepts a list of DNS look servers. Returns a vec or resolved IP addresses.
    fn lookup_active_nodes(dns: &[&str], port: u16) -> VecSocketAddr {
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
