use std;
use std::collections::HashMap;
use std::sync::Arc;
use futures::{self, Future, Stream};
use hyper;
use hyper::server::{Http, Request, Response as HyperResponse, Service as HyperService};
use serde::{Deserialize, Serialize};

use super::error::{Result, ResultExt};
use super::xmlfmt::{Value, CallValue, ResponseValue, error, parse, value};

pub struct HandlerError {
    pub code: i32,
    pub message: String,
}

impl HandlerError {
    fn new<T>(code: i32, message: T) -> HandlerError
    where
        T: Into<String>,
    {
        HandlerError {
            code,
            message: message.into(),
        }
    }
}

pub type HandlerResult = std::result::Result<Vec<Value>, HandlerError>;

impl Into<ResponseValue> for HandlerResult {
    fn into(self) -> ResponseValue {
        match self {
            Ok(params) => ResponseValue::Success { params },
            Err(HandlerError { code, message }) => ResponseValue::Fault { code, message },
        }
    }
}

type Handler = Box<Fn(Vec<Value>) -> HandlerResult + Send + Sync>;
type HandlerMap = HashMap<String, Handler>;

pub fn on_decode_fail(err: error::Error) -> HandlerResult {
    Err(HandlerError::new(
        400,
        format!("Failed to decode request: {}", err),
    ))
}

pub fn on_encode_fail(err: error::Error) -> HandlerResult {
    Err(HandlerError::new(
        500,
        format!("Failed to encode response: {}", err),
    ))
}

fn on_missing_method(_: Vec<Value>) -> HandlerResult {
    Err(HandlerError::new(404, "Requested method does not exist"))
}

pub struct Server {
    handlers: HandlerMap,
    on_missing_method: Handler,
}

impl Server {
    pub fn new() -> Server {
        Server {
            handlers: HashMap::new(),
            on_missing_method: Box::new(on_missing_method),
        }
    }

    pub fn register_value<K, T>(&mut self, name: K, handler: T)
    where
        K: Into<String>,
        T: Fn(Vec<Value>) -> HandlerResult + Send + Sync + 'static,
    {
        self.handlers.insert(name.into(), Box::new(handler));
    }

    pub fn register<'a, K, Treq, Tres, Thandler, Tef, Tdf>(
        &mut self,
        name: K,
        handler: Thandler,
        encode_fail: Tef,
        decode_fail: Tdf,
    ) where
        K: Into<String>,
        Treq: Deserialize<'a>,
        Tres: Serialize,
        Thandler: Fn(Treq) -> std::result::Result<Tres, HandlerError> + Send + Sync + 'static,
        Tef: Fn(error::Error) -> HandlerResult + Send + Sync + 'static,
        Tdf: Fn(error::Error) -> HandlerResult + Send + Sync + 'static,
    {
        self.register_value(name, move |req| {
            let params = match parse::params(req) {
                Ok(v) => v,
                Err(err) => return decode_fail(err),
            };
            let response = handler(params)?;
            match value::params(response) {
                Ok(v) => Ok(v),
                Err(err) => return encode_fail(err),
            }
        });
    }

    pub fn register_simple<'a, K, Treq, Tres, Thandler>(&mut self, name: K, handler: Thandler)
    where
        K: Into<String>,
        Treq: Deserialize<'a>,
        Tres: Serialize,
        Thandler: Fn(Treq) -> std::result::Result<Tres, HandlerError> + Send + Sync + 'static,
    {
        self.register(name, handler, on_encode_fail, on_decode_fail);
    }

    pub fn set_on_missing<T>(&mut self, handler: T)
    where
        T: Fn(Vec<Value>) -> HandlerResult + Send + Sync + 'static,
    {
        self.on_missing_method = Box::new(handler);
    }

    pub fn run(self, uri: &std::net::SocketAddr) -> Result<()> {
        let server = Arc::new(self);
        let server_call = Http::new()
            .bind(uri, move || Ok(Service { server: server.clone() }))
            .chain_err(|| "Failed to bind port")?;
        server_call.run().chain_err(|| "Failed to run server")?;
        Ok(())
    }

    fn handle(&self, req: CallValue) -> ResponseValue {
        self.handlers.get(&req.name).unwrap_or(
            &self.on_missing_method,
        )(req.params).into()
    }
}

struct Service {
    server: Arc<Server>,
}

impl HyperService for Service {
    type Request = Request;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = futures::AndThen<
        futures::Join<
            futures::stream::Concat2<hyper::Body>,
            futures::future::FutureResult<
                Arc<Server>,
                hyper::Error,
            >,
        >,
        futures::future::FutureResult<Self::Response, Self::Error>,
        fn((hyper::Chunk, Arc<Server>))
           -> futures::future::FutureResult<Self::Response, Self::Error>,
    >;

    fn call(&self, req: Request) -> Self::Future {
        req.body()
            .concat2()
            .join(futures::future::ok(self.server.clone()))
            .and_then(|(chunk, server)| {
                // TODO: use the right error type
                let call: CallValue = match parse::call_value(chunk.as_ref()) {
                    Ok(data) => data,
                    Err(_err) => return futures::future::err(hyper::error::Error::Incomplete),
                };
                let res = server.handle(call);
                let mut response = HyperResponse::new();
                response.set_body(format!("{}", res));
                futures::future::ok(response)

            })
    }
}
