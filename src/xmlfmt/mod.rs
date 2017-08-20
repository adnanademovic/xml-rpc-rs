use serde::{Deserialize, Serialize};

mod de;
pub mod error;
pub mod parse;
mod ser;
#[cfg(test)]
mod tests;
pub mod value;

pub use self::value::{Call, Fault, Params, Response, Value};

pub fn from_params<'a, T: Deserialize<'a>>(v: Params) -> error::Result<T> {
    parse::params(v)
}

pub fn into_params<T: Serialize>(v: T) -> error::Result<Params> {
    Ok(match v.serialize(ser::Serializer {})? {
        Value::Array(params) => params,
        data => vec![data],
    })
}
