use crate::XmlRpcResult;
use serde::{Deserialize, Serialize};

mod de;
pub mod parse;
mod ser;
#[cfg(test)]
mod tests;
pub mod value;

pub use self::value::{Call, Fault, Params, Response, Value};

pub fn from_params<'a, T: Deserialize<'a>>(mut params: Params) -> XmlRpcResult<T> {
    let data = if params.len() == 1 {
        params.pop().unwrap()
    } else {
        Value::Array(params)
    };

    let data = T::deserialize(data)?;
    Ok(data)
}

pub fn into_params<T: Serialize>(v: &T) -> XmlRpcResult<Params> {
    Ok(match v.serialize(ser::Serializer {})? {
        Value::Array(params) => params,
        data => vec![data],
    })
}
