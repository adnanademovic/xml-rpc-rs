use crate::Value;
use std::fmt::{Display, Formatter};
use xml::escape::escape_str_pcdata;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Request {
    pub name: String,
    pub params: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    Success { params: Vec<Value> },
    Failure { code: i32, message: String },
}

impl Response {
    pub fn success(params: Vec<Value>) -> Self {
        Response::Success { params }
    }

    pub fn failure(code: i32, message: String) -> Self {
        Response::Failure { code, message }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<?xml version=\"1.0\"?><methodCall><methodName>{}</methodName><params>",
            escape_str_pcdata(&self.name),
        )?;
        for param in &self.params {
            write!(f, "<param>{param}</param>")?;
        }
        write!(f, "</params></methodCall>")?;
        Ok(())
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Success { params } => {
                write!(f, "<?xml version=\"1.0\"?><methodResponse><params>")?;
                for param in params {
                    write!(f, "<param>{param}</param>")?;
                }
                write!(f, "</params></methodResponse>")?;
            }
            Response::Failure { code, message } => {
                write!(
                    f,
                    "<?xml version=\"1.0\"?>\
                        <methodResponse><fault><value><struct>\
                            <member><name>faultCode</name><value><int>{}</int></value></member>\
                            <member><name>faultString</name><value><string>{}</string></value></member>\
                        </struct></value></fault></methodResponse>",
                    code,
                    escape_str_pcdata(message),
                )?;
            }
        }
        Ok(())
    }
}
