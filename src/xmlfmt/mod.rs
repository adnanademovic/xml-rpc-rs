use serde::{Deserialize, Serialize};

mod de;
pub mod error;
pub mod parse;
mod ser;
#[cfg(test)]
mod tests;
pub mod value;

pub use self::value::{Call, Fault, Params, Response, Value};

pub fn from_params<'a, T: Deserialize<'a>>(mut params: Params) -> error::Result<T> {
    use self::error::ResultExt;
    let data = if params.len() == 1 {
        params.pop().unwrap()
    } else {
        Value::Array(params)
    };

    T::deserialize(data).chain_err(|| "Failed to convert XML-RPC to structure.")
}

pub fn into_params<T: Serialize>(v: &T) -> error::Result<Params> {
    Ok(match v.serialize(ser::Serializer {})? {
        Value::Array(params) => params,
        data => vec![data],
    })
}
