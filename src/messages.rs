use crate::util::literal_text_in_node;
use crate::Value;
use anyhow::{bail, Context};
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use roxmltree::{Document, Node};
use std::collections::HashMap;
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

fn read_params(node: Node) -> Vec<Value> {
    node.children()
        .find(|n| n.has_tag_name("params"))
        .map_or_else(Vec::new, |params_node| {
            params_node
                .children()
                .filter(|n| n.has_tag_name("param"))
                .filter_map(|param_node| {
                    param_node
                        .children()
                        .find_map(|value_node| Value::read_xml(value_node).ok())
                })
                .collect()
        })
}

impl Request {
    pub fn read_xml(data: &str) -> anyhow::Result<Self> {
        let document = Document::parse(data)?;
        let root = document.root();
        if !root.has_tag_name("methodCall") {
            bail!("Expected \"methodCall\" at the root of a request");
        }
        Ok(Self {
            name: literal_text_in_node(
                root.children()
                    .find(|child| child.has_tag_name("methodName"))
                    .context("\"methodName\" missing inside \"methodCall\"")?,
            )
            .into(),
            params: read_params(root),
        })
    }

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
    pub fn read_xml(data: &str) -> anyhow::Result<Self> {
        let document = Document::parse(data)?;
        let root = document.root();
        if !root.has_tag_name("methodResponse") {
            bail!("Expected \"methodResponse\" at the root of a request");
        }

        if let Some(fault) = root.children().find(|child| child.has_tag_name("fault")) {
            let fault_data = Value::read_xml(fault)?;
            let Value::Struct(fault_data) = fault_data else {
                bail!("Data inside \"fault\" needs to be a structure");
            };
            let mut fault_data = fault_data.into_iter().collect::<HashMap<String, Value>>();
            let Some(Value::Int(code)) = fault_data.remove("faultCode") else {
                bail!("Data inside \"fault\" requires a \"faultCode\" member with an integer type");
            };
            let Some(Value::String(message)) = fault_data.remove("faultString") else {
                bail!("Data inside \"fault\" requires a \"faultString\" member with a string type");
            };
            Ok(Self::failure(code, message))
        } else {
            Ok(Self::success(read_params(root)))
        }
    }

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
