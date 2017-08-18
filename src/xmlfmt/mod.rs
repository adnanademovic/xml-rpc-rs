mod de;
pub mod error;
#[cfg(test)]
mod tests;
mod value;

#[allow(unused_imports)]
use self::de::parse_xml;
pub use self::de::{parse_call, parse_response};
pub use self::value::{Call, Response, Value};
