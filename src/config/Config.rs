use std::fs::File;
use std::io::Read;
use serde_json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub socket_path: String
}

pub fn load_config() -> Configuration {
    let mut file = match File::open("/etc/charge_scmp/config.json") {
        Ok(file) => file,
        Err(e) => {
            panic!("{}", format!("Can't load configuration file! Error: {}", e));
        }
    };

    //Parse config file
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Error while reading config");

    let config: Configuration = match serde_json::from_str(contents.as_str()) {
        Ok(config) => config,
        Err(e) => {
            panic!("{}", format!("Error while reading: {}", e));
        }
    };

    return config;
}