use http::{StatusCode, Response};
use http_body_util::{StreamBody, BodyExt};
use hyper::body::Frame;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use futures_util::TryStreamExt;

use crate::traits::{HttpRequest, HttpResponse, StatusErr};


pub async fn handle(req: HttpRequest) -> HttpResponse {
    let file = File::open(req.uri().path()).await.map_err(StatusErr::not_found)?;

    let reader_stream = ReaderStream::new(file);

    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

    // Send response
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(boxed_body)
        .unwrap();

    // Ok(response)
    todo!()
}

