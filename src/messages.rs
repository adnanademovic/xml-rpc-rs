use crate::Value;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::io::Write;

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

fn write_declaration<W: Write>(writer: &mut Writer<W>) -> quick_xml::Result<()> {
    use quick_xml::events::{BytesDecl, Event};
    writer.write_event(Event::Decl(BytesDecl::new("1.0", None, None)))
}

impl Request {
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        write_declaration(writer)?;
        writer
            .create_element("methodCall")
            .write_inner_content(|writer| {
                writer
                    .create_element("methodName")
                    .write_text_content(BytesText::new(&self.name))?;
                writer
                    .create_element("params")
                    .write_inner_content(|writer| {
                        for param in &self.params {
                            writer
                                .create_element("param")
                                .write_inner_content(|writer| param.write_xml(writer))?;
                        }
                        Ok(())
                    })?;
                Ok(())
            })
            .map(|_| ())
    }
}

impl Response {
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        write_declaration(writer)?;
        writer
            .create_element("methodResponse")
            .write_inner_content(|writer| match self {
                Response::Success { params } => writer
                    .create_element("params")
                    .write_inner_content(|writer| {
                        for param in params {
                            writer
                                .create_element("param")
                                .write_inner_content(|writer| param.write_xml(writer))?;
                        }
                        Ok(())
                    })
                    .map(|_| ()),
                Response::Failure { code, message } => writer
                    .create_element("fault")
                    .write_inner_content(|writer| {
                        [
                            ("faultCode".into(), Value::from(*code)),
                            ("faultString".into(), Value::from(message.to_owned())),
                        ]
                        .into_iter()
                        .collect::<Value>()
                        .write_xml(writer)
                    })
                    .map(|_| ()),
            })
            .map(|_| ())
    }
}
