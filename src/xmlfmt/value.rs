use std;
use std::collections::HashMap;

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
pub struct Call {
    pub name: String,
    pub params: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    Success { params: Vec<Value> },
    Fault { code: i32, message: String },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Value::Int(v) => write!(f, "<value><i4>{}</i4></value>", v),
            Value::Bool(v) => {
                write!(
                    f,
                    "<value><boolean>{}</boolean></value>",
                    if v { 1 } else { 0 }
                )
            }
            Value::String(ref v) => write!(f, "<value><string>{}</string></value>", v),
            Value::Double(v) => write!(f, "<value><double>{}</double></value>", v),
            Value::DateTime(ref v) => {
                write!(
                    f,
                    "<value><dateTime.iso8601>{}</dateTime.iso8601></value>",
                    v
                )
            }
            Value::Base64(ref v) => write!(f, "<value><base64>{}</base64></value>", v),
            Value::Array(ref v) => {
                write!(f, "<value><array><data>")?;
                for item in v {
                    item.fmt(f)?;
                }
                write!(f, "</data></array></value>")
            }
            Value::Struct(ref v) => {
                write!(f, "<value><struct>")?;
                for (ref key, ref value) in v {
                    write!(f, "<member><name>{}</name>", key)?;
                    value.fmt(f)?;
                    write!(f, "</member>")?;
                }
                write!(f, "</struct></value>")
            }
        }
    }
}
