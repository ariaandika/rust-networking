
use http::{Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{service::service_fn, body::{Incoming, Bytes}, client::conn::http1::SendRequest};
use tokio::net::{TcpListener, ToSocketAddrs, TcpStream};

#[tokio::main]
async fn main() {
    server("127.0.0.1:3000").await.unwrap();
}

static SERVICE_UNAVAILABLE: &[u8] = b"Service Unavailable";


async fn server<T: ToSocketAddrs>(addr: T) -> Result<(), std::io::Error> {
    let tcp = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = tcp.accept().await?;
        let io = hyper_util::rt::TokioIo::new(stream);

        let server = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(dummy_handle));

        tokio::spawn(async move {
            if let Err(err) = server.await {
                eprintln!("Failed to serve request: {:?}", err);
            }
        });

    }
}

async fn dummy_handle(req: Request<Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, std::io::Error> {
    let proxy_addr = "127.0.0.1:8000";
    let tcp = TcpStream::connect(proxy_addr).await;

    let tcp = match tcp {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Failed to connect to proxy {proxy_addr}: {err}");
            let res: Response<BoxBody<Bytes, hyper::Error>> = Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Full::new(SERVICE_UNAVAILABLE.into()).map_err(|e|match e {}).boxed())
                .unwrap();
            return Ok(res);
        }
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





