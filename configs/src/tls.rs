use serde::Deserialize;

#[derive(Debug,Deserialize)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String
}
