use futures::{Future, Stream};
use hyper::{self, Client as HyperClient, Method, Request, Uri};
use serde::{Deserialize, Serialize};
use tokio_core::reactor::Core;
use super::error::{Result, ResultExt};
use super::xmlfmt::{Call, CallValue, Response, ResponseValue, parse};

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

    pub fn call<Treq, Tres>(&mut self, uri: Uri, req: Treq) -> Result<Tres>
    where
        Treq: Into<Req>,
        Res: Into<Result<Tres>>,
    {
        let mut request = Request::new(Method::Post, uri);
        request.set_body(req.into().data);
        let work = self.client.request(request).and_then(|res| {
            res.body().concat2().map(|chunk| chunk.to_vec())
        });
        let response = Res {
            data: self.core.run(work).chain_err(
                || "Failed to run the HTTP request within Tokio Core.",
            )?,
        };
        response.into()
    }
}

#[doc(hidden)]
pub struct Req {
    data: String,
}

impl From<CallValue> for Req {
    fn from(value: CallValue) -> Req {
        Req { data: format!("{}", value) }
    }
}

impl<T> From<Call<T>> for Req
where
    T: Serialize,
{
    fn from(value: Call<T>) -> Req {
        Req { data: format!("{}", value) }
    }
}

#[doc(hidden)]
pub struct Res {
    data: Vec<u8>,
}

impl Into<Result<ResponseValue>> for Res {
    fn into(self) -> Result<ResponseValue> {
        parse::response_value(self.data.as_slice()).map_err(Into::into)
    }
}

impl<'a, T> Into<Result<Response<T>>> for Res
where
    T: Deserialize<'a>,
{
    fn into(self) -> Result<Response<T>> {
        parse::response(self.data.as_slice()).map_err(Into::into)
    }
}
