extern crate serde;
extern crate toml;

use std::fs;

static mut CONFIG : Option<Config> = None;
static CONFIG_PATH : &'static str = "calamaris.toml";

#[derive(Deserialize)]
pub struct Config {
    db_ip: String
}

fn load() -> &'static Config {
    let cfg = fs::read_to_string(CONFIG_PATH).expect("Could not read config file");
    let parsed_cfg = toml::from_str(&cfg).expect("Could not parse config file");
    unsafe {
        CONFIG = Some(parsed_cfg);
    }
    return get();
}

pub fn get() -> &'static Config {
    unsafe {
        match &CONFIG {
            Some(u) => return &u,
            None => load(),
        }
    }
}
