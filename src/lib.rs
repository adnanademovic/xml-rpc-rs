#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_xml_rs;
extern crate tokio_core;

pub mod client;
pub mod error;
pub mod xmlfmt;
