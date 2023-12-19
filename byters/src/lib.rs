pub mod handler;
pub mod traits;
use std::fmt::Debug;

use hyper_util::rt::TokioIo;
use tokio::net::{ToSocketAddrs, TcpListener};
use traits::{StatusErr, HttpRequest, HttpResponse};

async fn resolve(req: HttpRequest) -> HttpResponse {
    let host = req
        .headers()
        .get("host")
        .ok_or_else(||StatusErr::bad_request("No host header"))?
        .to_str()
        .map_err(StatusErr::internal_err)?;

    match host {
        "localhost:3000" => handler::serve::handle(req).await,
        "localhost:3000" => handler::proxy::handle(req, "127.0.0.1:8000").await,
        _ => todo!()
    }
}

pub fn serve<A: ToSocketAddrs + Debug>(addr: A) {
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
                                match resolve(req).await {
                                    Ok(res) => Ok::<_,hyper::Error>(res),
                                    Err(err) => Ok(err),
                                }
                            }
                        })).await
                    {
                        eprintln!("Failed to serve request: {}", er);
                    }
                });
            }
    });
}


