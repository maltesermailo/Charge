use crate::{Cli, config};

pub fn daemon_main(cmd: Cli) {
    let config = config::load_config();

    println!("Starting Charge daemon...");
    println!("Config socket path: {}", config.socket_path);
}