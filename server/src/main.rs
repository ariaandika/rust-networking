
use std::{sync::{Arc, RwLock}, fmt::{Debug, Display}, time::Duration};

use http::{Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{service::service_fn, body::{Incoming, Bytes}, client::conn::http1::SendRequest};
use tokio::{net::{TcpListener, ToSocketAddrs, TcpStream}, signal::unix::SignalKind};

#[tokio::main]
async fn main() {
    server("127.0.0.1:3000").await.unwrap();
}

static SERVICE_UNAVAILABLE: &[u8] = b"Service Unavailable";

#[derive(Debug)]
struct AppState {
    #[allow(dead_code)]
    shutdown: bool
}

async fn server<T: ToSocketAddrs>(addr: T) -> Result<(), std::io::Error> {
    let tcp = TcpListener::bind(addr).await?;

    let config = Arc::new(RwLock::new(AppState { shutdown: false }));    

    signal_handler(config.clone());

    loop {
        let (stream, _) = tokio::select! {
            _ = tokio::signal::ctrl_c() => break shutdown(),
            s = tcp.accept() => s?
        };

        let io = hyper_util::rt::TokioIo::new(stream);

        let server = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(dummy_handle));

        tokio::spawn(async move {
            if let Err(err) = server.await {
                eprintln!("Failed to serve request: {:?}", err);
            }
        });
    };
    Ok(())
}

async fn dummy_handle(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::io::Error> {
    let proxy_addr = "127.0.0.1:8000";
    let tcp = TcpStream::connect(proxy_addr).await;

    let tcp = match tcp {
        Ok(t) => t,
        Err(err) => return service_unavailable(proxy_addr, err)
    };

    let io = hyper_util::rt::TokioIo::new(tcp);

    let (mut sender, conn): (SendRequest<Incoming>,_) = hyper::client::conn::http1::handshake(io).await.unwrap();

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


