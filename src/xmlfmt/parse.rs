use std;
use std::collections::HashMap;
use base64;
use serde::Deserialize;
use serde_xml_rs::deserialize;
use super::{Call, Response, Value, CallValue, ResponseValue};
use super::error::{Result, ResultExt};

#[allow(dead_code)]
pub fn xml<T: std::io::Read>(r: T) -> Result<Value> {
    let data: XmlValue = deserialize(r).chain_err(|| "Failed to parse XML-RPC data.")?;
    data.into()
}

pub fn call_value<T: std::io::Read>(r: T) -> Result<CallValue> {
    let data: XmlCall = deserialize(r).chain_err(|| "Failed to parse XML-RPC call.")?;
    data.into()
}

pub fn response_value<T: std::io::Read>(r: T) -> Result<ResponseValue> {
    let data: XmlResponse = deserialize(r).chain_err(
        || "Failed to parse XML-RPC response.",
    )?;
    data.into()
}

pub fn call<'a, T: std::io::Read, D: Deserialize<'a>>(r: T) -> Result<Call<D>> {
    let CallValue { name, mut params } = call_value(r)?;
    let data = if params.len() == 1 {
        params.pop().unwrap()
    } else {
        Value::Array(params)
    };
    Ok(Call {
        name: name,
        data: D::deserialize(data).chain_err(
            || "Failed to convert XML-RPC to structure.",
        )?,
    })
}

pub fn response<'a, T: std::io::Read, D: Deserialize<'a>>(r: T) -> Result<Response<D>> {
    match response_value(r)? {
        ResponseValue::Success { mut params } => {
            let data = if params.len() == 1 {
                params.pop().unwrap()
            } else {
                Value::Array(params)
            };

            Ok(Response::Success(D::deserialize(data).chain_err(
                || "Failed to convert XML-RPC to structure.",
            )?))
        }
        ResponseValue::Fault { code, message } => Ok(Response::Fault {
            code: code,
            message: message,
        }),
    }
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
            XmlValue::I4(v) => Value::Int(v),
            XmlValue::Int(v) => Value::Int(v),
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

impl Into<Result<CallValue>> for XmlCall {
    fn into(self) -> Result<CallValue> {
        let params: Result<Vec<Value>> = self.params.into();
        Ok(CallValue {
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

impl Into<Result<ResponseValue>> for XmlResponseResult {
    fn into(self) -> Result<ResponseValue> {
        match self {
            XmlResponseResult::Success(params) => {
                let params: Result<Vec<Value>> = params.into();
                Ok(ResponseValue::Success { params: params? })
            }
            XmlResponseResult::Failure { value: v } => {
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct FaultStruct {
                    #[serde(rename = "faultCode")]
                    pub code: i32,
                    #[serde(rename = "faultString")]
                    pub message: String,
                };

                let val: Result<Value> = v.into();
                let data = FaultStruct::deserialize(val?).chain_err(
                    || "Failed to decode fault structure",
                )?;

                Ok(ResponseValue::Fault {
                    code: data.code,
                    message: data.message,
                })

            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponse {
    #[serde(rename = "methodResponse")]
    Response(XmlResponseResult),
}

impl Into<Result<ResponseValue>> for XmlResponse {
    fn into(self) -> Result<ResponseValue> {
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
