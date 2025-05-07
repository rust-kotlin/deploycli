use std::{fs::File, io::Read, sync::LazyLock};

use serde::Deserialize;

const CONFIG_FILE: &str = "config.toml";

pub static CFG: LazyLock<Configs> = LazyLock::new(Configs::init);

impl Configs {
    pub fn init() -> Self {
        let mut file = match File::open(CONFIG_FILE) {
            Ok(f) => f,
            Err(e) => {
                panic!(
                    "Configuration file does not exist: {}, error message: {}",
                    CONFIG_FILE, e
                )
            }
        };
        let mut cfg_contents = String::new();
        match file.read_to_string(&mut cfg_contents) {
            Ok(s) => s,
            Err(e) => panic!("Failed to read configuration file, error message: {}", e),
        };
        match toml::from_str(&cfg_contents) {
            Ok(c) => c,
            Err(e) => panic!("Failed to parse configuration file, error message: {}", e),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Configs {
    pub server: Server,
    pub log: Log,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub address: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub filter_level: String,
    pub with_ansi: bool,
    pub to_stdout: bool,
    pub directory: String,
    pub file_name: String,
    pub rolling: String,
}
