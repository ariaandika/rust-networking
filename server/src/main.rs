
use std::{sync::{Arc, RwLock}, fmt::{Debug, Display}, collections::HashMap};

use http::{Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{service::service_fn, body::{Incoming, Bytes}, client::conn::http1::SendRequest};
use tokio::{net::{TcpListener, ToSocketAddrs, TcpStream}, signal::unix::SignalKind};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let addr = if std::env::var("TLS").is_ok() { "[::]" } else { "127.0.0.1" };
    let port1 = std::env::var("PORT").unwrap_or("3000".into());

    let server1 = server(format!("{addr}:{port1}"));
    let server2 = {
        match std::env::var("PORT2") {
            Ok(port2) => Some(server_redirect(format!("{addr}:{port2}"))),
            Err(_) => None,
        }
    };

    let f = tokio::join!(
        server1,
        async { if let Some(s) = server2 { s.await } else { Ok(()) } }
    );

    f.0?;
    f.1?;

    Ok(())
}

static SERVICE_UNAVAILABLE: &[u8] = b"Service Unavailable";

#[derive(Clone, Debug)]
struct AppState {
    domains: HashMap<String,DomainConfig>
}

impl Default for AppState {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert("deuzo.me".into(), DomainConfig { target: "localhost:8000".into() });
        map.insert("api.deuzo.me".into(), DomainConfig { target: "localhost:5000".into() });
        Self {
            domains: map
        }
    }
}

#[derive(Clone, Debug)]
struct DomainConfig {
    target: String,
}

async fn server<T: ToSocketAddrs>(addr: T) -> Result<(), std::io::Error> {
    let tcp = TcpListener::bind(addr).await?;

    let config = Arc::new(RwLock::new(AppState::default()));    

    signal_handler(config.clone());

    loop {
        let (stream, _) = tokio::select! {
            _ = tokio::signal::ctrl_c() => break shutdown(),
            s = tcp.accept() => s?
        };

        let io = hyper_util::rt::TokioIo::new(stream);
        let config = Arc::clone(&config);
        let server = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    let config = Arc::clone(&config);
                    async move {
                        dummy_handle(req, config).await
                    }
                }));

        tokio::spawn(async move {
            if let Err(err) = server.await {
                eprintln!("Failed to serve request: {:?}", err);
            }
        });
    };
    Ok(())
}

async fn server_redirect<T: ToSocketAddrs>(addr: T) -> Result<(), std::io::Error> {
    let tcp = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            s = tcp.accept() => s?
        };

        let io = hyper_util::rt::TokioIo::new(stream);
        let server = hyper::server::conn::http1::Builder::new()
                .keep_alive(false)
                .serve_connection(io, service_fn(redirect)) ;

        tokio::spawn(async move {
            if let Err(err) = server.await {
                eprintln!("Failed to serve request: {:?}", err);
            }
        });
    };
    Ok(())
}

async fn redirect(_: Request<Incoming>) -> Result<Response<String>, std::io::Error> {
    let res = Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "deuzo.me")
        .header("Conection", "close")
        .body(String::new())
        .unwrap();
    Ok(res)
}

async fn dummy_handle(req: Request<Incoming>, config: Arc<RwLock<AppState>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::io::Error> {

    let host = match req.headers().get("host") {
        Some(host) => match host.to_str() {
            Ok(h) => h,
            Err(_) => return bad_request(),
        },
        None => return bad_request()
    };

    let proxy_addr = {
        let target = config.read().expect("Deadlock");
        match target.domains.get(host) {
            Some(c) => {
                let t = &c.target;
                t.clone()
            }
            None => return not_found()
        }
    };

    let tcp = TcpStream::connect(&proxy_addr).await;

    let tcp = match tcp {
        Ok(t) => t,
        Err(err) => return service_unavailable(&proxy_addr, err)
    };

    let io = hyper_util::rt::TokioIo::new(tcp);

    let (mut sender, conn): (SendRequest<Incoming>,_) = match hyper::client::conn::http1::handshake(io).await {
        Ok(c) => c,
        Err(err) => return service_unavailable(proxy_addr, err),
    };

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Failed to connect to proxy: {:?}", err);
        }
    });

    let res = sender
        .send_request(req)
        .await
        .unwrap()
        .map(|b|b.boxed());

    Ok(res)
}

fn service_unavailable<T: Display, Err: Display>(proxy_addr: T, err: Err) -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::io::Error> {
    eprintln!("Failed to connect to proxy {proxy_addr}: {err}");
    let res: Response<BoxBody<Bytes, hyper::Error>> = Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(Full::new(SERVICE_UNAVAILABLE.into()).map_err(|e|match e {}).boxed())
        .unwrap();
    Ok(res)
}

fn bad_request() -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::io::Error> {
    let res: Response<BoxBody<Bytes, hyper::Error>> = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::default()).map_err(|e|match e {}).boxed())
        .unwrap();
    Ok(res)
}

fn not_found() -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::io::Error> {
    let res: Response<BoxBody<Bytes, hyper::Error>> = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::default()).map_err(|e|match e {}).boxed())
        .unwrap();
    Ok(res)
}

fn shutdown() {
    println!("Shutting down, may take time to wait in flight connection...");
}

fn signal_handler(config: Arc<RwLock<AppState>>) {
    tokio::spawn(async move {
        let mut usr1 = tokio::signal::unix::signal(SignalKind::user_defined1()).unwrap();
        tokio::select! {
            _ = usr1.recv() => {
                println!("Reloading config...");
                signal_handler(config)
            }
        }
    });
}


