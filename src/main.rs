use clap::Parser;
use crate::daemon::daemon_main;
use crate::supervisor::supervisor_main;

#[path = "config/Config.rs"]
mod config;
mod daemon;
mod supervisor;

#[derive(Parser)]
#[command(version)]
struct Cli {
    //Whether to run in daemon mode or immediate supervisor mode
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    daemon: bool

    //The next arguments are ignored on daemon mode

}

fn main() {
    let cli = Cli::parse();

    if cli.daemon {
        daemon_main()
        //Switch into daemon.rs
    } else {
        //Switch into supervisor.rs and parse rest arguments
        supervisor_main()
    }
}
