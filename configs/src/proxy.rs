#![allow(unused)]

use std::{collections::HashMap, fs::{self, File}, io::{BufReader, self}};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug,Deserialize)]
pub struct Config {
    domains: HashMap<String,DomainConfig>
}

#[derive(Debug,Deserialize)]
pub struct DomainConfig {
    proxy: Option<ProxyConfig>,
    serve: Option<ServeConfig>,
}


#[derive(Debug,Deserialize)]
pub struct ProxyConfig {
    target: String
}

#[derive(Debug,Deserialize)]
pub struct ServeConfig {
    root: String,
    path: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct TlsConfig {
    cert: String,
    key: String
}

impl Config {
    pub fn setup() -> std::io::Result<Config> {
        let mut dirs = vec!["./config.json", "../config.json", "../../config.json"];
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


