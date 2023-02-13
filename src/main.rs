use log::error;
use p2p_node_handshake::Config;

#[tokio::main]
async fn main() {
    let env = env_logger::Env::default().filter_or("log-level-info", "info");
    env_logger::init_from_env(env);

    let config = Config::build(std::env::args()).unwrap_or_else(|err| {
        error!("Problem parsing arguments: {err:?}");
        std::process::exit(1);
    });

    if let Err(e) = p2p_node_handshake::run(&config).await {
        error!("Application error:\n{e:?}");
        std::process::exit(2);
    }
}
