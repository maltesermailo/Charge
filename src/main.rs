#[path = "config/Config.rs"]
mod config;

fn main() {
    config::load_config();
    println!("Hello, world!");
}
