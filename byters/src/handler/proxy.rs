



use http::Response;
use hyper::{client::conn::http1::SendRequest, body::Incoming};
use tokio::net::ToSocketAddrs;

use http_body_util::BodyExt;
use tokio::net::TcpStream;

use crate::traits::{StatusErr, HttpResponse, HttpRequest};

pub async fn handle<A>(req: HttpRequest, addr: A) -> HttpResponse
where
    A: ToSocketAddrs,
{
    let stream = TcpStream::connect(addr)
        .await
        .map_err(StatusErr::proxy_failed)?;

    let io = hyper_util::rt::TokioIo::new(stream);

    let (mut sender, conn): (SendRequest<Incoming>, _) = hyper::client::conn::http1::handshake(io)
                             .await
                             .map_err(StatusErr::proxy_failed)?;

    tokio::spawn(async {
        if let Err(e) = conn.await {
            eprintln!("Failed to connect: {}", e);
        }
    });

    let res: Response<Incoming> = sender.send_request(req).await.map_err(StatusErr::proxy_failed)?;

    Ok(res.map(BodyExt::boxed))
}

