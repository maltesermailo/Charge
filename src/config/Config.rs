use std::fs::File;
use std::io::Read;

struct Configuration {
    socket_path: String
}

pub(crate) fn load_config() -> Result<Configuration, Err> {
    let mut file = match File::open("/etc/charge_scmp/config.json") {
        Ok(file) => file,
        None => {
            panic!("Can't load configuration file!");
        }
    };

    //Parse config file
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Error while reading config");

    //Parse rest
    return Ok(Configuration{ socket_path: "empty".to_string() });
}