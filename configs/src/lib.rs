use std::{collections::HashMap, fs::File, io::{self, BufReader}};
use serde::Deserialize;

pub mod proxy;
pub mod tls;

#[derive(Debug,Deserialize)]
pub struct Config {
    pub domains: HashMap<String,proxy::DomainConfig>
}

impl Config {
    pub fn setup() -> std::io::Result<Config> {
        let mut dirs = vec!["./config.json", "../config.json"];
        let mut file = File::open(dirs.pop().unwrap());

        while file.is_err() {
            if let Some(dir) = dirs.pop() {
                file = File::open(dir);
            } else {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Failed to read config file"));
            }
        }

        let file = file?;
        let reader = BufReader::new(file);
        let app: Config = serde_json::from_reader(reader)?;
        Ok(app)
    }

    pub fn reload(&mut self, config: Self) {
        *self = config;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn app() {
        println!("{:?}", Config::setup());
    }
}
