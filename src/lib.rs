use futures::future;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, StatusCode};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[macro_use]
extern crate log;

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

/// Simple wrapper for running metricbeat provider servers
/// Based on Hyper
pub fn run<T: Serialize + Send + 'static>(addr: &SocketAddr, data: Arc<Mutex<T>>)  {
    let service = move || {
        let data = data.clone();
        service_fn(move |_: Request<Body>| -> BoxFut {
            let mut response = Response::new(Body::empty());
            let data = data.clone();
            let guard = data.lock().unwrap();
            if let Ok(json) = serde_json::to_string(&*guard) {
                response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                *response.body_mut() = Body::from(json);
            } else {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
            Box::new(future::ok(response))
        })
    };
    let server = Server::bind(addr)
        .serve(service)
        .map_err(|e| error!("Metricbeat server error: {}", e));
    info!("Metricbeat listening on http://{}", addr);
    hyper::rt::run(server);
}
