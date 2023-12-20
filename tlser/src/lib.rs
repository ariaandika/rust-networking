use std::{fs, io::BufReader, sync::Arc};

use tokio_rustls::{rustls::{pki_types::{CertificateDer, PrivateKeyDer}, ServerConfig}, TlsAcceptor};

fn load_certs(filename: String) -> Vec<CertificateDer<'static>> {
    let file = fs::File::open(filename).expect("Cannot read file");
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .map(|e|e.unwrap())
        .collect()
}

fn load_keys(filename: String) -> PrivateKeyDer<'static> {
    let file = fs::File::open(filename.clone()).expect("Cannot read file");
    let mut reader = BufReader::new(file);
    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => return key.into(),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => return key.into(),
            Some(rustls_pemfile::Item::Sec1Key(key)) => return key.into(),
            None => break,
            _ => {}
        }
    }

    panic!("No keys found in {}", filename);
}

pub type TlsAcceptorType = TlsAcceptor;

pub fn setup_tls(cert: String, key: String) -> TlsAcceptor {
    let cert = load_certs(cert);
    let key = load_keys(key);

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .expect("Cannot verify certificate");

    let acc = TlsAcceptor::from(Arc::new(config));
    acc
}

