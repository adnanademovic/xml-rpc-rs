use std;
use std::collections::HashMap;
use std::sync::Arc;
use futures::{self, Future, Stream};
use hyper;
use hyper::server::{Http, NewService as HyperNewService, Request, Response as HyperResponse,
                    Service as HyperService};
use serde::{Deserialize, Serialize};

use super::error::{Result, ResultExt};
use super::xmlfmt::{error, from_params, into_params, parse, Call, Fault, Response, Value};

type Handler = Box<Fn(Vec<Value>) -> Response + Send + Sync>;
type HandlerMap = HashMap<String, Handler>;

pub fn on_decode_fail(err: &error::Error) -> Response {
    Err(Fault::new(
        400,
        format!("Failed to decode request: {}", err),
    ))
}

pub fn on_encode_fail(err: &error::Error) -> Response {
    Err(Fault::new(
        500,
        format!("Failed to encode response: {}", err),
    ))
}

fn on_missing_method(_: Vec<Value>) -> Response {
    Err(Fault::new(404, "Requested method does not exist"))
}

pub struct Server {
    handlers: HandlerMap,
    on_missing_method: Handler,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            handlers: HashMap::new(),
            on_missing_method: Box::new(on_missing_method),
        }
    }
}

impl Server {
    pub fn new() -> Server {
        Server::default()
    }

    pub fn register_value<K, T>(&mut self, name: K, handler: T)
    where
        K: Into<String>,
        T: Fn(Vec<Value>) -> Response + Send + Sync + 'static,
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
        Thandler: Fn(Treq) -> std::result::Result<Tres, Fault> + Send + Sync + 'static,
        Tef: Fn(&error::Error) -> Response + Send + Sync + 'static,
        Tdf: Fn(&error::Error) -> Response + Send + Sync + 'static,
    {
        self.register_value(name, move |req| {
            let params = match from_params(req) {
                Ok(v) => v,
                Err(err) => return decode_fail(&err),
            };
            let response = handler(params)?;
            into_params(&response).or_else(|v| encode_fail(&v))
        });
    }

    pub fn register_simple<'a, K, Treq, Tres, Thandler>(&mut self, name: K, handler: Thandler)
    where
        K: Into<String>,
        Treq: Deserialize<'a>,
        Tres: Serialize,
        Thandler: Fn(Treq) -> std::result::Result<Tres, Fault> + Send + Sync + 'static,
    {
        self.register(name, handler, on_encode_fail, on_decode_fail);
    }

    pub fn set_on_missing<T>(&mut self, handler: T)
    where
        T: Fn(Vec<Value>) -> Response + Send + Sync + 'static,
    {
        self.on_missing_method = Box::new(handler);
    }

    pub fn bind(self, uri: &std::net::SocketAddr) -> Result<BoundServer> {
        Http::new()
            .bind(uri, NewService::new(self))
            .chain_err(|| "Failed to bind port")
            .map(BoundServer::new)
    }

    fn handle(&self, req: Call) -> Response {
        self.handlers
            .get(&req.name)
            .unwrap_or(&self.on_missing_method)(req.params)
    }
}

pub struct BoundServer {
    server: hyper::Server<NewService, hyper::Body>,
}

impl BoundServer {
    fn new(server: hyper::Server<NewService, hyper::Body>) -> BoundServer {
        BoundServer { server }
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.server
            .local_addr()
            .chain_err(|| "Failed to get socket address")
    }

    pub fn run(self) -> Result<()> {
        self.server.run().chain_err(|| "Failed to run server")
    }

    pub fn run_until<F>(self, shutdown_signal: F) -> Result<()>
    where
        F: futures::Future<Item = (), Error = ()>,
    {
        self.server
            .run_until(shutdown_signal)
            .chain_err(|| "Failed to run server")
    }
}

struct Service {
    server: Arc<Server>,
}

type BodyFuture = futures::stream::Concat2<hyper::Body>;
type ServerResultFuture = futures::future::FutureResult<Arc<Server>, hyper::Error>;
type BodyAndServerFuture = futures::Join<BodyFuture, ServerResultFuture>;
type ResponseResultFuture = futures::future::FutureResult<HyperResponse, hyper::Error>;
type ChunkServerResponder = fn((hyper::Chunk, Arc<Server>)) -> ResponseResultFuture;

impl HyperService for Service {
    type Request = Request;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = futures::AndThen<BodyAndServerFuture, ResponseResultFuture, ChunkServerResponder>;

    fn call(&self, req: Request) -> Self::Future {
        req.body()
            .concat2()
            .join(futures::future::ok(Arc::clone(&self.server)))
            .and_then(|(chunk, server)| {
                use super::xmlfmt::value::ToXml;
                // TODO: use the right error type
                let call: Call = match parse::call(chunk.as_ref()) {
                    Ok(data) => data,
                    Err(_err) => return futures::future::err(hyper::error::Error::Incomplete),
                };
                let res = server.handle(call);
                let body = res.to_xml();
                futures::future::ok(
                    HyperResponse::new()
                        .with_header(hyper::header::ContentLength(body.len() as u64))
                        .with_header(hyper::header::ContentType::xml())
                        .with_body(body),
                )
            })
    }
}

struct NewService {
    server: Arc<Server>,
}

impl NewService {
    fn new(server: Server) -> NewService {
        NewService {
            server: Arc::new(server),
        }
    }
}

impl HyperNewService for NewService {
    type Request = Request;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Instance = Service;

    fn new_service(&self) -> std::io::Result<Self::Instance> {
        Ok(Service {
            server: Arc::clone(&self.server),
        })
    }
}
