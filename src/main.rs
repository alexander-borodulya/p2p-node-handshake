mod dns_seed_mananger;

use dns_seed_mananger::DnsSeedManager;

#[tokio::main]
async fn main() {
    let dsm = DnsSeedManager::new_with_default_dns_seeds().await;
    println!("dsm: {}", dsm.ip_count());
    println!("dsm.get(0): {:?}", dsm.get(0));
    
    let dsm = DnsSeedManager::default();
    println!("dsm: {}", dsm.ip_count());
    println!("dsm.get(0): {:?}", dsm.get(0));
}
