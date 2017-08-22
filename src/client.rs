use std;
use futures::{Future, Stream};
use hyper::{self, Client as HyperClient, Method, Request, Uri};
use serde::{Deserialize, Serialize};
use tokio_core::reactor::Core;
use super::error::{Result, ResultExt};
use super::xmlfmt::{Call, Fault, Params, Response, parse, from_params, into_params};

pub struct Client {
    core: Core,
    client: HyperClient<hyper::client::HttpConnector>,
}

impl Client {
    pub fn new() -> Result<Client> {
        let core = Core::new().chain_err(|| "Failed to initialize Tokio Core.")?;
        let client = HyperClient::new(&core.handle());
        Ok(Client {
            core: core,
            client: client,
        })
    }

    pub fn call_value<Tkey>(&mut self, uri: &Uri, name: Tkey, params: Params) -> Result<Response>
    where
        Tkey: Into<String>,
    {
        use super::xmlfmt::value::ToXml;
        let mut request = Request::new(Method::Post, uri.clone());
        request.set_body(
            Call {
                name: name.into(),
                params,
            }.to_xml(),
        );
        let work = self.client.request(request).and_then(|res| {
            res.body().concat2().map(|chunk| chunk.to_vec())
        });
        let response = self.core.run(work).chain_err(
            || "Failed to run the HTTP request within Tokio Core.",
        )?;
        parse::response(response.as_slice()).map_err(Into::into)
    }

    pub fn call<'a, Tkey, Treq, Tres>(
        &mut self,
        uri: &Uri,
        name: Tkey,
        req: Treq,
    ) -> Result<std::result::Result<Tres, Fault>>
    where
        Tkey: Into<String>,
        Treq: Serialize,
        Tres: Deserialize<'a>,
    {
        match self.call_value(uri, name, into_params(&req)?) {
            Ok(Ok(v)) => from_params(v).map(Ok).map_err(Into::into),
            Ok(Err(v)) => Ok(Err(v)),
            Err(v) => Err(v),
        }
    }
}
