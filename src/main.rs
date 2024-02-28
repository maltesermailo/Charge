use std::mem::MaybeUninit;
use clap::Parser;
use crate::daemon::daemon_main;
use crate::supervisor::{seccomp_notif, supervisor_main};

#[path = "config/Config.rs"]
mod config;
mod daemon;
mod supervisor;
mod utils;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    //Whether to run in daemon mode or immediate supervisor mode
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    daemon: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    test: bool,

    //The next arguments are ignored on daemon mode
    #[arg(long, help = "File descriptor ID of Seccomp channel for supervisor mode", required = false, default_value = "0")]
    fd: u32,
    #[arg(long, short, required = false, default_value = "0")]
    pid: u32,
    #[arg(long, short, required = false, default_value = "noid")]
    id: String,
    #[arg(long = "containerName", short, required = false, default_value = "")]
    container_name: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.daemon {
        daemon_main(cli)
        //Switch into daemon.rs
    } else {
        //Switch into supervisor.rs and parse rest arguments
        supervisor_main(cli)
    }

    Ok(())
}
