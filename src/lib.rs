#![recursion_limit = "1024"]

#[macro_use]
extern crate lazy_static;
pub extern crate rouille;

pub mod client;
mod error;
pub mod server;
mod xmlfmt;

pub use client::{call, call_value};
pub use error::{XmlRpcError, XmlRpcResult};
pub use server::Server;
pub use xmlfmt::{from_params, into_params, Call, Fault, Params, Response, Value};
