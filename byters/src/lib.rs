use std::convert::Infallible;
use std::pin::Pin;
use std::task::Poll;

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

pub async fn handle<A>(req: Request<Incoming>, addr: A) -> Result<Respon,Response<()>>
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

    while let Some(n) = res.frame().await {
        n;
    };

    todo!()

    // let res_result = sender
    //     .send_request(req)
    //     .await
    //     .map_err(StatusErr::hy_internal_err)?
    //     .map(BoxBody::new);
    //
    // Ok(res_result)
}

struct StatusErr;
impl StatusErr {
    fn internal_err(er: std::io::Error) -> Response<()> {
        eprintln!("Internal server error: {:?}",er);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(())
            .unwrap()
    }
    fn hy_internal_err(er: hyper::Error) ->  Response<()> {
        eprintln!("Internal server error: {:?}",er);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(())
            .unwrap()
    }
}


trait StdErr where Self: Sized {
    fn to_err(self) -> (Self,StatusCode);
}

enum Si<D> {
    Ok(D),
    Oof
}

struct App<D> {
    own: Si<D>
}

impl<D: Buf> Body for App<D> {
    type Data = D;

    type Error = anyhow::Error;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match self.own {
            Si::Ok(d) => d.poll_frame(cx),
            Si::Oof => todo!(),
        }
    }

    fn is_end_stream(&self) -> bool {
        true
    }

    fn size_hint(&self) -> hyper::body::SizeHint {
        hyper::body::SizeHint::with_exact(0)
    }
}


#[cfg(test)]
mod tests {
    use http_body_util::Empty;
    use hyper::{server::conn::http1::Builder, service::service_fn};
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpListener;
    use super::*;

    #[test]
    fn as_hyper_server_service() {
        tokio::spawn(async {
            let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(stream);
                
                tokio::spawn(async move {
                    Builder::new()
                        .serve_connection(io, service_fn(hand))
                });
            }
        });
    }

    async fn map_it(req: Request<Incoming>) -> Response<BoxBody<Bytes, hyper::Error>> {
        let rs = match handle(req, "127.0.0.1:3000").await {
            Ok(r) => r,
            Err((status,msg)) => {
                let fuck = Response::builder()
                .status(status)
                .body(BoxBody { ..Default::default() })
                // .body(BoxBody::default())
                .unwrap();
                fuck
            },
        };

        todo!()
    }

    async fn hand(_: Request<Incoming>) -> Result<Response<Empty<&'static [u8]>>, anyhow::Error> {

        todo!()
        // Ok(Response::builder().body("Snive".into()).unwrap())
    }
}


