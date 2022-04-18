use super::{Call, Fault, Response, Value};
use crate::{XmlRpcError, XmlRpcResult};
use base64;
use regex::Regex;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std;
use std::collections::HashMap;
use std::num::ParseFloatError;

fn wrap_in_string(content: String) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"<value\s*/>").unwrap();
        static ref RE2: Regex = Regex::new(r"<value\s*>\s*<string\s*/>\s*</value\s*>").unwrap();
        static ref RE3: Regex = Regex::new(r"<value\s*>(?P<rest>[^<>]*)</value\s*>").unwrap();
    }
    RE3.replace_all(
        &RE2.replace_all(
            &RE1.replace_all(&content, "<value><string></string></value>"),
            "<value><string></string></value>",
        ),
        "<value><string>$rest</string></value>",
    )
    .into()
}

#[allow(dead_code)]
pub fn xml<T: std::io::Read>(mut r: T) -> XmlRpcResult<Value> {
    let mut content = String::new();
    r.read_to_string(&mut content)?;
    let data: XmlValue = from_str(&wrap_in_string(content))?;
    data.into()
}

pub fn call<T: std::io::Read>(mut r: T) -> XmlRpcResult<Call> {
    let mut content = String::new();
    r.read_to_string(&mut content)?;
    let data: XmlCall = from_str(&wrap_in_string(content))?;
    data.into()
}

pub fn response<T: std::io::Read>(mut r: T) -> XmlRpcResult<Response> {
    let mut content = String::new();
    r.read_to_string(&mut content)?;
    let data: XmlResponse = from_str(&wrap_in_string(content))?;
    data.into()
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlValue {
    #[serde(rename = "i4")]
    I4(i32),
    #[serde(rename = "int")]
    Int(i32),
    #[serde(rename = "boolean")]
    Bool(i32),
    #[serde(rename = "string")]
    Str(String),
    #[serde(rename = "double")]
    Double(String),
    #[serde(rename = "dateTime.iso8601")]
    DateTime(String),
    #[serde(rename = "base64")]
    Base64(String),
    #[serde(rename = "array")]
    Array(XmlArray),
    #[serde(rename = "struct")]
    Struct(XmlStruct),
}

impl From<XmlValue> for XmlRpcResult<Value> {
    fn from(src: XmlValue) -> Self {
        Ok(match src {
            XmlValue::I4(v) | XmlValue::Int(v) => Value::Int(v),
            XmlValue::Bool(v) => Value::Bool(v != 0),
            XmlValue::Str(v) => Value::String(v),
            XmlValue::Double(v) => Value::Double(
                v.parse()
                    .map_err(|err: ParseFloatError| XmlRpcError::Decoding(err.to_string()))?,
            ),
            XmlValue::DateTime(v) => Value::DateTime(v),
            XmlValue::Base64(v) => Value::Base64(
                base64::decode(v.as_bytes())
                    .map_err(|err| XmlRpcError::Decoding(err.to_string()))?,
            ),
            XmlValue::Array(v) => {
                let items: XmlRpcResult<Vec<Value>> = v.into();
                Value::Array(items?)
            }
            XmlValue::Struct(v) => {
                let items: XmlRpcResult<HashMap<String, Value>> = v.into();
                Value::Struct(items?)
            }
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "methodCall")]
struct XmlCall {
    #[serde(rename = "methodName")]
    pub name: String,
    pub params: XmlParams,
}

impl From<XmlCall> for XmlRpcResult<Call> {
    fn from(src: XmlCall) -> Self {
        let params: XmlRpcResult<Vec<Value>> = src.params.into();
        Ok(Call {
            name: src.name,
            params: params?,
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponseResult {
    #[serde(rename = "params")]
    Success(XmlParams),
    #[serde(rename = "fault")]
    Failure { value: XmlValue },
}

impl From<XmlResponseResult> for XmlRpcResult<Response> {
    fn from(src: XmlResponseResult) -> Self {
        match src {
            XmlResponseResult::Success(params) => {
                let params: XmlRpcResult<Vec<Value>> = params.into();
                Ok(Ok(params?))
            }
            XmlResponseResult::Failure { value: v } => {
                let val: XmlRpcResult<Value> = v.into();

                Ok(Err(Fault::deserialize(val?)?))
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponse {
    #[serde(rename = "methodResponse")]
    Response(XmlResponseResult),
}

impl From<XmlResponse> for XmlRpcResult<Response> {
    fn from(src: XmlResponse) -> Self {
        match src {
            XmlResponse::Response(v) => v.into(),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParams {
    #[serde(rename = "param", default)]
    pub params: Vec<XmlParamData>,
}

impl From<XmlParams> for XmlRpcResult<Vec<Value>> {
    fn from(src: XmlParams) -> Self {
        src.params.into_iter().map(From::from).collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParamData {
    pub value: XmlValue,
}

impl From<XmlParamData> for XmlRpcResult<Value> {
    fn from(src: XmlParamData) -> Self {
        src.value.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArray {
    #[serde(rename = "data")]
    pub data: XmlArrayData,
}

impl From<XmlArray> for XmlRpcResult<Vec<Value>> {
    fn from(src: XmlArray) -> Self {
        src.data.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArrayData {
    #[serde(default)]
    pub value: Vec<XmlValue>,
}

impl From<XmlArrayData> for XmlRpcResult<Vec<Value>> {
    fn from(src: XmlArrayData) -> Self {
        src.value.into_iter().map(From::from).collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStruct {
    #[serde(rename = "member", default)]
    pub members: Vec<XmlStructItem>,
}

impl From<XmlStruct> for XmlRpcResult<HashMap<String, Value>> {
    fn from(src: XmlStruct) -> Self {
        src.members.into_iter().map(From::from).collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStructItem {
    pub name: String,
    pub value: XmlValue,
}

impl From<XmlStructItem> for XmlRpcResult<(String, Value)> {
    fn from(src: XmlStructItem) -> Self {
        let value: XmlRpcResult<Value> = src.value.into();
        Ok((src.name, value?))
    }
}
