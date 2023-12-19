use std::fmt::Debug;

use bytes::Bytes;
use http::{Response, StatusCode};
use http_body_util::combinators::BoxBody;
use hyper::body::Incoming;

pub type Respon<Er = hyper::Error> = Response<BoxBody<Bytes, Er>>;
pub type HttpResponse<Er = hyper::Error> = Result<Respon<Er>, Respon<Er>>;
pub type HttpRequest = http::Request<Incoming>;

pub struct StatusErr;
impl StatusErr {
    pub fn internal_err<T: Debug>(er: T) -> Respon {
        eprintln!("Internal server error: {:?}",er);
        Self::res(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn not_found<T: Debug,Er: Debug>(_: T) -> Respon<Er> {
        Self::res(StatusCode::NOT_FOUND)
    }

    pub fn bad_request<T: Debug>(_: T) -> Respon {
        Self::res(StatusCode::BAD_REQUEST)
    }

    pub fn proxy_failed<T: Debug>(er: T) -> Respon {
        eprintln!("Failed to connect proxy server: {:?}",er);
        Self::res(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn res<Er: Debug>(status: StatusCode) -> Respon<Er> {
        Response::builder()
            .status(status)
            .body(BoxBody::default())
            .expect("Idk the fk why this can error")
    }
}
