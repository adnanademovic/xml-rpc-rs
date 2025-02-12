use super::error::{Result, ResultExt};
use super::{Call, Fault, Response, Value};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use std;
use std::collections::HashMap;

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
pub fn xml<T: std::io::Read>(mut r: T) -> Result<Value> {
    let mut content = String::new();
    r.read_to_string(&mut content)
        .chain_err(|| "Failed to read data source.")?;
    let data: XmlValue = serde_xml_rs::from_str(&wrap_in_string(content))
        .chain_err(|| "Failed to parse XML-RPC data.")?;
    data.into()
}

pub fn call<T: std::io::Read>(mut r: T) -> Result<Call> {
    let mut content = String::new();
    r.read_to_string(&mut content)
        .chain_err(|| "Failed to read data source.")?;
    let data: XmlCall = serde_xml_rs::from_str(&wrap_in_string(content))
        .chain_err(|| "Failed to parse XML-RPC call.")?;
    data.into()
}

pub fn response<T: std::io::Read>(mut r: T) -> Result<Response> {
    let mut content = String::new();
    r.read_to_string(&mut content)
        .chain_err(|| "Failed to read data source.")?;
    let data: XmlResponse = serde_xml_rs::from_str(&wrap_in_string(content))
        .chain_err(|| "Failed to parse XML-RPC response.")?;
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

impl From<XmlValue> for Result<Value> {
    fn from(val: XmlValue) -> Self {
        Ok(match val {
            XmlValue::I4(v) | XmlValue::Int(v) => Value::Int(v),
            XmlValue::Bool(v) => Value::Bool(v != 0),
            XmlValue::Str(v) => Value::String(v),
            XmlValue::Double(v) => Value::Double(v.parse().chain_err(|| "Failed to parse double")?),
            XmlValue::DateTime(v) => Value::DateTime(v),
            XmlValue::Base64(v) => Value::Base64(
                STANDARD
                    .decode(v.as_bytes())
                    .chain_err(|| "Failed to parse base64")?,
            ),
            XmlValue::Array(v) => {
                let items: Result<Vec<Value>> = v.into();
                Value::Array(items?)
            }
            XmlValue::Struct(v) => {
                let items: Result<HashMap<String, Value>> = v.into();
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

impl From<XmlCall> for Result<Call> {
    fn from(val: XmlCall) -> Self {
        let params: Result<Vec<Value>> = val.params.into();
        Ok(Call {
            name: val.name,
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

impl From<XmlResponseResult> for Result<Response> {
    fn from(val: XmlResponseResult) -> Self {
        match val {
            XmlResponseResult::Success(params) => {
                let params: Result<Vec<Value>> = params.into();
                Ok(Ok(params?))
            }
            XmlResponseResult::Failure { value: v } => {
                use serde::Deserialize;

                let val: Result<Value> = v.into();

                Ok(Err(
                    Fault::deserialize(val?).chain_err(|| "Failed to decode fault structure")?
                ))
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponse {
    #[serde(rename = "methodResponse")]
    Response(XmlResponseResult),
}

impl From<XmlResponse> for Result<Response> {
    fn from(val: XmlResponse) -> Self {
        match val {
            XmlResponse::Response(v) => v.into(),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParams {
    #[serde(rename = "param", default)]
    pub params: Vec<XmlParamData>,
}

impl From<XmlParams> for Result<Vec<Value>> {
    fn from(val: XmlParams) -> Self {
        val.params
            .into_iter()
            .map(Into::<Result<Value>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParamData {
    pub value: XmlValue,
}

impl From<XmlParamData> for Result<Value> {
    fn from(val: XmlParamData) -> Self {
        val.value.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArray {
    #[serde(rename = "data")]
    pub data: XmlArrayData,
}

impl From<XmlArray> for Result<Vec<Value>> {
    fn from(val: XmlArray) -> Self {
        val.data.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArrayData {
    #[serde(default)]
    pub value: Vec<XmlValue>,
}

impl From<XmlArrayData> for Result<Vec<Value>> {
    fn from(val: XmlArrayData) -> Self {
        val.value
            .into_iter()
            .map(Into::<Result<Value>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStruct {
    #[serde(rename = "member", default)]
    pub members: Vec<XmlStructItem>,
}

impl From<XmlStruct> for Result<HashMap<String, Value>> {
    fn from(val: XmlStruct) -> Self {
        val.members
            .into_iter()
            .map(Into::<Result<(String, Value)>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStructItem {
    pub name: String,
    pub value: XmlValue,
}

impl From<XmlStructItem> for Result<(String, Value)> {
    fn from(val: XmlStructItem) -> Self {
        let value: Result<Value> = val.value.into();
        Ok((val.name, value?))
    }
}
