use std;
use std::collections::HashMap;
use serde::Serialize;
use super::ser::Serializer;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    String(String),
    Double(f64),
    DateTime(String),
    Base64(String),
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallValue {
    pub name: String,
    pub params: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call<T> {
    pub name: String,
    pub data: T,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResponseValue {
    Success { params: Vec<Value> },
    Fault { code: i32, message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Response<T> {
    Success(T),
    Fault { code: i32, message: String },
}

impl<T> std::fmt::Display for Call<T>
where
    T: Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = self.data.serialize(Serializer {}).map_err(
            |_| std::fmt::Error,
        )?;
        match data {
            Value::Array(params) => {
                CallValue {
                    name: self.name.clone(),
                    params,
                }
            }
            _ => CallValue {
                name: self.name.clone(),
                params: vec![data],
            },
        }.fmt(f)
    }
}

impl std::fmt::Display for CallValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, include_str!("templates/call_1.xml"), name = self.name)?;
        for param in &self.params {
            write!(f, "<param>{}</param>", param)?;
        }
        write!(f, include_str!("templates/call_2.xml"))
    }
}

impl<T> std::fmt::Display for Response<T>
where
    T: Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ref value = match *self {
            Response::Fault { code, ref message } => {
                return ResponseValue::Fault {
                    code,
                    message: message.clone(),
                }.fmt(f)
            }
            Response::Success(ref v) => v,
        };
        let data = value.serialize(Serializer {}).map_err(|_| std::fmt::Error)?;
        match data {
            Value::Array(params) => ResponseValue::Success { params },
            _ => ResponseValue::Success { params: vec![data] },
        }.fmt(f)
    }
}

impl std::fmt::Display for ResponseValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        match *self {
            ResponseValue::Success { ref params } => {
                write!(f, include_str!("templates/response_success_1.xml"))?;
                for param in params {
                    write!(f, "<param>{}</param>", param)?;
                }
                write!(f, include_str!("templates/response_success_2.xml"))
            }
            ResponseValue::Fault { code, ref message } => {
                write!(
                    f,
                    include_str!("templates/response_fault.xml"),
                    code = code,
                    message = message
                )
            }
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<value>")?;
        match *self {
            Value::Int(v) => write!(f, "<i4>{}</i4>", v),
            Value::Bool(v) => write!(f, "<boolean>{}</boolean>", if v { 1 } else { 0 }),
            Value::String(ref v) => write!(f, "<string>{}</string>", v),
            Value::Double(v) => write!(f, "<double>{}</double>", v),
            Value::DateTime(ref v) => write!(f, "<dateTime.iso8601>{}</dateTime.iso8601>", v),
            Value::Base64(ref v) => write!(f, "<base64>{}</base64>", v),
            Value::Array(ref v) => {
                write!(f, "<array><data>")?;
                for item in v {
                    item.fmt(f)?;
                }
                write!(f, "</data></array>")
            }
            Value::Struct(ref v) => {
                write!(f, "<struct>")?;
                for (ref key, ref value) in v {
                    write!(f, "<member><name>{}</name>{}</member>", key, value)?;
                }
                write!(f, "</struct>")
            }
        }?;
        write!(f, "</value>")
    }
}
