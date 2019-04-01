use base64;
use serde::de::Unexpected;
use std;
use std::collections::HashMap;
use xml::escape::escape_str_pcdata;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    String(String),
    Double(f64),
    DateTime(String),
    Base64(Vec<u8>),
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
}

impl Value {
    pub fn unexpected(&self) -> Unexpected {
        match *self {
            Value::Int(v) => Unexpected::Signed(i64::from(v)),
            Value::Bool(v) => Unexpected::Bool(v),
            Value::String(ref v) => Unexpected::Str(v),
            Value::Double(v) => Unexpected::Float(v),
            Value::DateTime(_) => Unexpected::Other("dateTime.iso8601"),
            Value::Base64(ref v) => Unexpected::Bytes(v),
            Value::Array(_) => Unexpected::Seq,
            Value::Struct(_) => Unexpected::Map,
        }
    }
}

pub type Params = Vec<Value>;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Fault {
    #[serde(rename = "faultCode")]
    pub code: i32,
    #[serde(rename = "faultString")]
    pub message: String,
}

impl Fault {
    pub fn new<T>(code: i32, message: T) -> Fault
    where
        T: Into<String>,
    {
        Fault {
            code,
            message: message.into(),
        }
    }
}

pub type Response = std::result::Result<Params, Fault>;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub name: String,
    pub params: Params,
}

pub trait ToXml {
    fn to_xml(&self) -> String;
}

impl ToXml for Call {
    fn to_xml(&self) -> String {
        format!(
            include_str!("templates/call.xml"),
            name = self.name,
            params = self
                .params
                .iter()
                .map(|param| format!("<param>{}</param>", param.to_xml()))
                .collect::<String>()
        )
    }
}

impl ToXml for Response {
    fn to_xml(&self) -> String {
        match *self {
            Ok(ref params) => format!(
                include_str!("templates/response_success.xml"),
                params = params
                    .iter()
                    .map(|param| format!("<param>{}</param>", param.to_xml()))
                    .collect::<String>()
            ),
            Err(Fault { code, ref message }) => format!(
                include_str!("templates/response_fault.xml"),
                code = code,
                message = message
            ),
        }
    }
}

impl ToXml for Value {
    fn to_xml(&self) -> String {
        match *self {
            Value::Int(v) => format!("<value><i4>{}</i4></value>", v),
            Value::Bool(v) => format!(
                "<value><boolean>{}</boolean></value>",
                if v { 1 } else { 0 }
            ),
            Value::String(ref v) => {
                format!("<value><string>{}</string></value>", escape_str_pcdata(v))
            }
            Value::Double(v) => format!("<value><double>{}</double></value>", v),
            Value::DateTime(ref v) => {
                format!("<value><dateTime.iso8601>{}</dateTime.iso8601></value>", v)
            }
            Value::Base64(ref v) => {
                format!("<value><base64>{}</base64></value>", base64::encode(v))
            }
            Value::Array(ref v) => format!(
                "<value><array><data>{}</data></array></value>",
                v.iter().map(Value::to_xml).collect::<String>()
            ),
            Value::Struct(ref v) => format!(
                "<value><struct>{}</struct></value>",
                v.iter()
                    .map(|(key, value)| format!(
                        "<member><name>{}</name>{}</member>",
                        key,
                        value.to_xml()
                    ))
                    .collect::<String>()
            ),
        }
    }
}
