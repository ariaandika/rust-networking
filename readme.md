# Rust Networking

collection of rust library for networking

libs:
- bytes
- http
- http-body
- http-body-util
- rustls, rustls-pemfile
- hyper
- hyper-util
- tokio
- tokio-rustls
- serde, serde_json

packages:

## byters

use for managing handlers, getting familier with its structs and traits

- bytes, main types for generic in returned response
- http, types for http
- http-body-util, `box` a response generic type, allows for multiple return type
- hyper, http request and response parser
- hyper, converting handler function to `Service`
- hyper-util, creating tokio io
- tokio, async runtime
- tokio, tcp module

## Problem!

`hyper::Incoming` implement boxbody as `BoxBody<Bytes, hyper::Error>`

`StreamBody` implement boxbody as `BoxBody<Bytes, std::io::Error>`

so the returned response have different type

still doesnt know how to map error inside `BoxBody`


