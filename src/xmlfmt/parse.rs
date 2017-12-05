use std;
use std::collections::HashMap;
use base64;
use serde_xml_rs::deserialize;
use super::{Call, Fault, Response, Value};
use super::error::{Result, ResultExt};

#[allow(dead_code)]
pub fn xml<T: std::io::Read>(r: T) -> Result<Value> {
    let data: XmlValue = deserialize(r).chain_err(|| "Failed to parse XML-RPC data.")?;
    data.into()
}

pub fn call<T: std::io::Read>(r: T) -> Result<Call> {
    let data: XmlCall = deserialize(r).chain_err(|| "Failed to parse XML-RPC call.")?;
    data.into()
}

pub fn response<T: std::io::Read>(r: T) -> Result<Response> {
    let data: XmlResponse = deserialize(r).chain_err(
        || "Failed to parse XML-RPC response.",
    )?;
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

impl Into<Result<Value>> for XmlValue {
    fn into(self) -> Result<Value> {
        Ok(match self {
            XmlValue::I4(v) | XmlValue::Int(v) => Value::Int(v),
            XmlValue::Bool(v) => Value::Bool(v != 0),
            XmlValue::Str(v) => Value::String(v),
            XmlValue::Double(v) => Value::Double(v.parse().chain_err(|| "Failed to parse double")?),
            XmlValue::DateTime(v) => Value::DateTime(v),
            XmlValue::Base64(v) => Value::Base64(base64::decode(v.as_bytes()).chain_err(
                || "Failed to parse base64",
            )?),
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

impl Into<Result<Call>> for XmlCall {
    fn into(self) -> Result<Call> {
        let params: Result<Vec<Value>> = self.params.into();
        Ok(Call {
            name: self.name,
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

impl Into<Result<Response>> for XmlResponseResult {
    fn into(self) -> Result<Response> {
        match self {
            XmlResponseResult::Success(params) => {
                let params: Result<Vec<Value>> = params.into();
                Ok(Ok(params?))
            }
            XmlResponseResult::Failure { value: v } => {
                use serde::Deserialize;

                let val: Result<Value> = v.into();

                Ok(Err(Fault::deserialize(val?).chain_err(
                    || "Failed to decode fault structure",
                )?))
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponse {
    #[serde(rename = "methodResponse")]
    Response(XmlResponseResult),
}

impl Into<Result<Response>> for XmlResponse {
    fn into(self) -> Result<Response> {
        match self {
            XmlResponse::Response(v) => v.into(),
        }
    }
}


#[derive(Debug, PartialEq, Deserialize)]
struct XmlParams {
    #[serde(rename = "param")]
    pub params: Vec<XmlParamData>,
}

impl Into<Result<Vec<Value>>> for XmlParams {
    fn into(self) -> Result<Vec<Value>> {
        self.params
            .into_iter()
            .map(Into::<Result<Value>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParamData {
    pub value: XmlValue,
}

impl Into<Result<Value>> for XmlParamData {
    fn into(self) -> Result<Value> {
        self.value.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArray {
    #[serde(rename = "data")]
    pub data: XmlArrayData,
}

impl Into<Result<Vec<Value>>> for XmlArray {
    fn into(self) -> Result<Vec<Value>> {
        self.data.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArrayData {
    #[serde(default)]
    pub value: Vec<XmlValue>,
}

impl Into<Result<Vec<Value>>> for XmlArrayData {
    fn into(self) -> Result<Vec<Value>> {
        self.value
            .into_iter()
            .map(Into::<Result<Value>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStruct {
    #[serde(rename = "member")]
    pub members: Vec<XmlStructItem>,
}

impl Into<Result<HashMap<String, Value>>> for XmlStruct {
    fn into(self) -> Result<HashMap<String, Value>> {
        self.members
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

impl Into<Result<(String, Value)>> for XmlStructItem {
    fn into(self) -> Result<(String, Value)> {
        let value: Result<Value> = self.value.into();
        Ok((self.name, value?))
    }
}
