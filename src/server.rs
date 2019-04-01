use rouille;
use serde::{Deserialize, Serialize};
use std;
use std::collections::HashMap;

use super::error::{ErrorKind, Result};
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

    pub fn bind(
        self,
        uri: &std::net::SocketAddr,
    ) -> Result<BoundServer<impl Fn(&rouille::Request) -> rouille::Response + Send + Sync + 'static>>
    {
        rouille::Server::new(uri, move |req| self.handle_outer(req))
            .map_err(|err| ErrorKind::BindFail(err.description().into()).into())
            .map(BoundServer::new)
    }

    fn handle_outer(&self, request: &rouille::Request) -> rouille::Response {
        use super::xmlfmt::value::ToXml;

        let body = match request.data() {
            Some(data) => data,
            None => return rouille::Response::empty_400(),
        };

        // TODO: use the right error type
        let call: Call = match parse::call(body) {
            Ok(data) => data,
            Err(_err) => return rouille::Response::empty_400(),
        };
        let res = self.handle(call);
        let body = res.to_xml();
        rouille::Response::from_data("text/xml", body)
    }

    fn handle(&self, req: Call) -> Response {
        self.handlers
            .get(&req.name)
            .unwrap_or(&self.on_missing_method)(req.params)
    }
}

pub struct BoundServer<F>
where
    F: Send + Sync + 'static + Fn(&rouille::Request) -> rouille::Response,
{
    server: rouille::Server<F>,
    // server: hyper::Server<NewService, hyper::Body>,
}

impl<F> BoundServer<F>
where
    F: Send + Sync + 'static + Fn(&rouille::Request) -> rouille::Response,
{
    fn new(server: rouille::Server<F>) -> Self {
        Self { server }
    }

    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.server.server_addr()
    }

    pub fn run(self) {
        self.server.run()
    }

    pub fn poll(&self) {
        self.server.poll()
    }
}
