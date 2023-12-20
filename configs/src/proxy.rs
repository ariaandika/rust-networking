#![allow(warnings)]

use std::{fs::File, io::{BufReader, self}};
use serde::Deserialize;

use crate::tls::TlsConfig;


#[derive(Debug,Deserialize)]
pub struct DomainConfig {
    pub proxy: Option<ProxyConfig>,
    pub serve: Option<ServeConfig>,
    pub tls: Option<TlsConfig>
}


#[derive(Debug,Clone,Deserialize)]
pub struct ProxyConfig {
    pub target: String
}

#[derive(Debug,Deserialize)]
pub struct ServeConfig {
    pub root: String,
    pub path: Option<String>,
}

