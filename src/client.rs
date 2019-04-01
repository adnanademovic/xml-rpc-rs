use super::error::{Result, ResultExt};
use super::xmlfmt::{from_params, into_params, parse, Call, Fault, Params, Response};
use hyper::{self, Client as HyperClient};
use serde::{Deserialize, Serialize};
use std;
use Url;

use hyper::header::Headers;
header! { (ContentType, "ContentType") => [String] }

pub fn call_value<Tkey>(uri: &Url, name: Tkey, params: Params) -> Result<Response>
where
    Tkey: Into<String>,
{
    Client::new()?.call_value(uri, name, params)
}

pub fn call<'a, Tkey, Treq, Tres>(
    uri: &Url,
    name: Tkey,
    req: Treq,
) -> Result<std::result::Result<Tres, Fault>>
where
    Tkey: Into<String>,
    Treq: Serialize,
    Tres: Deserialize<'a>,
{
    Client::new()?.call(uri, name, req)
}

pub struct Client {
    client: HyperClient,
}

impl Client {
    pub fn new() -> Result<Client> {
        let client = HyperClient::new();
        Ok(Client { client: client })
    }

    pub fn call_value<Tkey>(&mut self, uri: &Url, name: Tkey, params: Params) -> Result<Response>
    where
        Tkey: Into<String>,
    {
        use super::xmlfmt::value::ToXml;
        let body_str = Call {
            name: name.into(),
            params,
        }
        .to_xml();
        let bytes: &[u8] = body_str.as_bytes();
        let body = hyper::client::Body::BufBody(bytes, bytes.len());

        let mut headers = Headers::new();
        headers.set(ContentType("xml".to_owned()));

        let response = self
            .client
            .post(uri.as_ref())
            .headers(headers)
            .body(body)
            .send()
            .chain_err(|| "Failed to run the HTTP request within hyper.")?;

        parse::response(response).map_err(Into::into)
    }

    pub fn call<'a, Tkey, Treq, Tres>(
        &mut self,
        uri: &Url,
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
