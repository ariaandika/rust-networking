use std::fmt::Debug;

use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use http_body_util::combinators::BoxBody;

use hyper::client::conn::http1::SendRequest;
use hyper::{self, body::Incoming};
use hyper_util;

use hyper_util::rt::TokioIo;
use tokio::net::{TcpStream, ToSocketAddrs, TcpListener};

pub type Respon = Response<BoxBody<Bytes, hyper::Error>>;

pub async fn handle<A>(req: Request<Incoming>, addr: A) -> Result<Respon,Response<BoxBody<Bytes, hyper::Error>>>
where
    A: ToSocketAddrs
{
    let stream = TcpStream::connect(addr)
        .await
        .map_err(StatusErr::internal_err)?;

    let io = hyper_util::rt::TokioIo::new(stream);

    let (mut sender, conn): (SendRequest<Incoming>, _) = hyper::client::conn::http1::handshake(io)
                             .await
                             .map_err(StatusErr::internal_err)?;

    tokio::spawn(async {
        if let Err(e) = conn.await {
            eprintln!("conn error: {}", e);
        }
    });

    let res = sender.send_request(req).await.map_err(StatusErr::internal_err)?;

    Ok(res.map(BodyExt::boxed))
}

pub async fn serve<A: ToSocketAddrs + Debug>(addr: A) {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .expect("Failed to build tokio runtime")
        .block_on(async {
            let listener = TcpListener::bind(&addr)
                .await
                .expect(format!("Failed to bind {:?}", &addr).as_str());

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(er) => {
                        eprintln!("Failed to accept connection: {:?}", er);
                        continue;
                    },
                };
                let io = TokioIo::new(stream);
                
                tokio::spawn(async move {
                    if let Err(er) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, hyper::service::service_fn(|req|{
                            async move {
                                match handle(req, "127.0.0.1:8000").await {
                                    Ok(res) => Ok::<_,hyper::Error>(res),
                                    Err(err) => Ok(err),
                                }
                            }
                        })).await
                    {
                        eprintln!("Cannot serve request: {}", er);
                    }
                });
            }
    });
}

struct StatusErr;
impl StatusErr {
    fn internal_err<T: Debug>(er: T) -> Response<BoxBody<Bytes, hyper::Error>> {
        eprintln!("Internal server error: {:?}",er);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(BoxBody::default())
            .expect("Idk the fk why this can error")
    }
}

