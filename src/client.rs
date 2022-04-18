use super::error::{Result, ResultExt};
use super::xmlfmt::{from_params, into_params, parse, Call, Fault, Params, Response};
use serde::{Deserialize, Serialize};
use std;

pub fn call_value<Tkey>(uri: &str, name: Tkey, params: Params) -> Result<Response>
where
    Tkey: Into<String>,
{
    use super::xmlfmt::value::ToXml;
    let body_str = Call {
        name: name.into(),
        params,
    }
    .to_xml();

    let response = ureq::post(uri)
        .set("Content-Type", "text/xml")
        .send_string(&body_str)
        .chain_err(|| "Failed to run the HTTP request within ureq.")?
        .into_reader();

    parse::response(response).map_err(Into::into)
}

pub fn call<'a, Tkey, Treq, Tres>(
    uri: &str,
    name: Tkey,
    req: Treq,
) -> Result<std::result::Result<Tres, Fault>>
where
    Tkey: Into<String>,
    Treq: Serialize,
    Tres: Deserialize<'a>,
{
    match call_value(uri, name, into_params(&req)?) {
        Ok(Ok(v)) => from_params(v).map(Ok).map_err(Into::into),
        Ok(Err(v)) => Ok(Err(v)),
        Err(v) => Err(v),
    }
}
