/// DNS Seeds
/// 
/// From: https://github.com/bitcoin/bitcoin/blob/v24.0.1/src/chainparams.cpp#L123
/// 
/// "seed.bitcoin.sipa.be."          // Pieter Wuille, only supports x1, x5, x9, and xd
/// "dnsseed.bluematt.me."           // Matt Corallo, only supports x9
/// "dnsseed.bitcoin.dashjr.org."    // Luke Dashjr
/// "seed.bitcoinstats.com."         // Christian Decker, supports x1 - xf
/// "seed.bitcoin.jonasschnelli.ch." // Jonas Schnelli, only supports x1, x5, x9, and xd
/// "seed.btc.petertodd.org."        // Peter Todd, only supports x1, x5, x9, and xd
/// "seed.bitcoin.sprovoost.nl."     // Sjors Provoost
/// "dnsseed.emzy.de."               // Stephan Oeste
/// "seed.bitcoin.wiz.biz."          // Jason Maurice
/// 

use std::net;

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

#[derive(Debug)]
pub struct DnsSeedManager {
    pub seeds: Vec<std::net::SocketAddr>,
}

impl Default for DnsSeedManager {
    fn default() -> Self {
        Self { 
            seeds: DnsSeedManager::lookup_dns_seeds(&DEFAULT_DNS_SEEDS, DEFAULT_PORT_MAINNET)
        }
    }
}

impl DnsSeedManager {
    pub fn new() -> Self {
        Self {
            seeds: Vec::new(),
        }
    }

    pub fn ip_count(&self) -> usize {
        self.seeds.len()
    }

    pub fn get(&self, i: usize) -> Option<&net::SocketAddr> {
        self.seeds.get(i)
    }

    pub async fn new_with_default_dns_seeds() -> Self {
        let mut dsm = DnsSeedManager::new();
        let dns_seed_addr = (DEFAULT_DNS_SEEDS[0], DEFAULT_PORT_MAINNET);
        match tokio::net::lookup_host(dns_seed_addr).await {
            Ok(addr) => {
                dsm.seeds.extend(addr.collect::<Vec<std::net::SocketAddr>>());
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        dsm
    }

    #[allow(dead_code)]
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
