use std;
use futures::{self, Future, Stream};
use hyper;
use hyper::server::{Http, Request, Response as HyperResponse, Service as HyperService};

use super::error::{Result, ResultExt};
use super::xmlfmt::{CallValue, ResponseValue, parse};

pub struct Server;

impl Server {
    pub fn run(uri: &std::net::SocketAddr) -> Result<()> {
        let server = Http::new().bind(uri, || Ok(Service)).chain_err(
            || "Failed to bind port",
        )?;
        server.run().chain_err(|| "Failed to run server")?;
        Ok(())
    }
}

struct Service;

impl HyperService for Service {
    type Request = Request;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = futures::AndThen<
        futures::stream::Concat2<hyper::Body>,
        futures::future::FutureResult<Self::Response, Self::Error>,
        fn(hyper::Chunk)
           -> futures::future::FutureResult<Self::Response, Self::Error>,
    >;

    fn call(&self, req: Request) -> Self::Future {
        req.body().concat2().and_then(|chunk| {
            // TODO: use the right error type
            let call: CallValue = match parse::call_value(chunk.as_ref()) {
                Ok(data) => data,
                Err(_err) => return futures::future::err(hyper::error::Error::Incomplete),
            };
            let res = handler(call);
            let mut response = HyperResponse::new();
            response.set_body(format!("{}", res));
            futures::future::ok(response)

        })
    }
}

fn handler(req: CallValue) -> ResponseValue {
    ResponseValue::Success { params: req.params }
}
