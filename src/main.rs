use std::mem::MaybeUninit;
use clap::Parser;
use crate::daemon::daemon_main;
use crate::supervisor::{seccomp_notif, supervisor_main};

#[path = "config/Config.rs"]
mod config;
mod daemon;
mod supervisor;

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
    fd: u32
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut seccomp_notif_uninit: MaybeUninit<seccomp_notif> = unsafe { MaybeUninit::zeroed() };
    println!("RUST NOTIF SIZE: {}", std::mem::size_of::<seccomp_notif>());
    println!("RUST NOTIF ALIGNMENT: {}", std::mem::align_of::<seccomp_notif>());

    if cli.daemon {
        daemon_main(cli)
        //Switch into daemon.rs
    } else {
        //Switch into supervisor.rs and parse rest arguments
        supervisor_main(cli)
    }
}
