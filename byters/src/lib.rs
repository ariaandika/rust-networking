#![allow(unused)]

use std::convert::Infallible;
use std::pin::Pin;
use std::task::Poll;

use anyhow::anyhow;
use bytes::{Bytes, Buf};
use http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use http_body_util::combinators::BoxBody;

use hyper::body::Body;
use hyper::client::conn::http1::SendRequest;
use hyper::{self, body::Incoming};
use hyper_util;

use tokio::net::{TcpStream, ToSocketAddrs};

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
                             .map_err(StatusErr::hy_internal_err)?;

    tokio::spawn(async {
        if let Err(e) = conn.await {
            eprintln!("conn error: {}", e);
        }
    });

    let mut res = sender.send_request(req).await.map_err(StatusErr::hy_internal_err)?;

    Ok(res.map(BodyExt::boxed))
}

struct StatusErr;
impl StatusErr {
    fn internal_err(er: std::io::Error) -> Response<BoxBody<Bytes, hyper::Error>> {
        eprintln!("Internal server error: {:?}",er);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(BoxBody::default())
            .unwrap()
    }
    fn hy_internal_err(er: hyper::Error) -> Response<BoxBody<Bytes, hyper::Error>> {
        eprintln!("Internal server error: {:?}",er);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(BoxBody::default())
            .unwrap()
    }
}


trait StdErr where Self: Sized {
    fn to_err(self) -> (Self,StatusCode);
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use http_body_util::Empty;
    use hyper::{server::conn::http1::Builder, service::service_fn};
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;
    use super::*;

    #[test]
    fn as_hyper_server_service() {
        let mut rt = tokio::runtime::Builder::new_current_thread();
        rt.enable_io();
        rt.build().unwrap().block_on(async {
            let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                
                tokio::spawn(async move {
                    if let Err(er) = Builder::new()
                        .serve_connection(io, service_fn(|req|{
                            async move {
                                match handle(req, "127.0.0.1:8000").await {
                                    Ok(res) => Ok::<_,anyhow::Error>(res),
                                    Err(err) => Ok(err),
                                }
                            }
                        })).await
                    {
                        eprintln!("Cannot serve request: {}", er);
                    }
                });
            }
        })
    }
}


