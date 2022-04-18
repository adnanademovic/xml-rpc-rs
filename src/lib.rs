#![recursion_limit = "1024"]

pub mod client;
mod error;
pub mod server;
mod xmlfmt;

pub use client::{call, call_value};
pub use error::{XmlRpcError, XmlRpcResult};
pub use rouille::{Request as RouilleRequest, Response as RouilleResponse};
pub use server::Server;
pub use xmlfmt::{from_params, into_params, Call, Fault, Params, Response, Value};
