use crate::util::literal_text_in_node;
use crate::Value;
use anyhow::{bail, Context};
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use roxmltree::{Document, Node};
use std::collections::HashMap;
use std::io::{Cursor, Write};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Request {
    pub name: String,
    pub params: Vec<Value>,
}

impl Request {
    pub fn new(name: String, params: Vec<Value>) -> Self {
        Self { name, params }
    }
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
        let root = document
            .root()
            .children()
            .find(|node| node.has_tag_name("methodCall"))
            .context("Expected \"methodCall\" at the root of a request")?;

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

    pub fn write_xml<W: Write>(&self, w: W) -> anyhow::Result<()> {
        let mut writer = Writer::new(w);
        self.write_xml_with_writer(&mut writer)?;
        Ok(())
    }

    pub fn write_xml_string(&self) -> anyhow::Result<String> {
        let mut cursor = Cursor::new(Vec::new());
        self.write_xml(&mut cursor)?;
        Ok(String::from_utf8(cursor.into_inner())?)
    }

    fn write_xml_with_writer<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
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
        let root = document
            .root()
            .children()
            .find(|node| node.has_tag_name("methodResponse"))
            .context("Expected \"methodResponse\" at the root of a request")?;

        if let Some(fault) = root.children().find(|child| child.has_tag_name("fault")) {
            let fault_data = fault
                .children()
                .find_map(|n| Value::read_xml(n).ok())
                .context("Expected valid value inside fault element")?;
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

    pub fn write_xml<W: Write>(&self, w: W) -> anyhow::Result<()> {
        let mut writer = Writer::new(w);
        self.write_xml_with_writer(&mut writer)?;
        Ok(())
    }

    pub fn write_xml_string(&self) -> anyhow::Result<String> {
        let mut cursor = Cursor::new(Vec::new());
        self.write_xml(&mut cursor)?;
        Ok(String::from_utf8(cursor.into_inner())?)
    }

    pub fn write_xml_with_writer<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_success_response() {
        assert_eq!(
            Response::success(vec![
                Value::String("South Dakota".into()),
                Value::Struct(vec![
                    ("foo".into(), Value::Int(42)),
                    ("bar".into(), Value::String("baz".into())),
                    ("bar2".into(), Value::String("baz2".into())),
                ]),
            ])
            .write_xml_string()
            .unwrap(),
            "<?xml version=\"1.0\"?>\
            <methodResponse>\
                <params>\
                    <param>\
                        <value><string>South Dakota</string></value>\
                    </param>\
                    <param>\
                        <value>\
                            <struct>\
                                <member>\
                                    <name>foo</name>\
                                    <value><int>42</int></value>\
                                </member>\
                                <member>\
                                    <name>bar</name>\
                                    <value><string>baz</string></value>\
                                </member>\
                                <member>\
                                    <name>bar2</name>\
                                    <value><string>baz2</string></value>\
                                </member>\
                            </struct>\
                        </value>\
                    </param>\
                </params>\
            </methodResponse>",
        );
    }

    #[test]
    fn encodes_failure_response() {
        assert_eq!(
            Response::failure(4, "Too many parameters.".into())
                .write_xml_string()
                .unwrap(),
            "<?xml version=\"1.0\"?>\
            <methodResponse>\
                <fault>\
                    <value>\
                        <struct>\
                            <member>\
                                <name>faultCode</name>\
                                <value><int>4</int></value>\
                            </member>\
                            <member>\
                                <name>faultString</name>\
                                <value><string>Too many parameters.</string></value>\
                            </member>\
                        </struct>\
                    </value>\
                </fault>\
            </methodResponse>",
        );
    }

    #[test]
    fn encodes_request() {
        assert_eq!(
            Request::new(
                "foobar".into(),
                vec![
                    Value::String("South Dakota".into()),
                    Value::Struct(vec![
                        ("foo".into(), Value::Int(42)),
                        ("bar".into(), Value::String("baz".into())),
                    ])
                ]
            )
            .write_xml_string()
            .unwrap(),
            "<?xml version=\"1.0\"?>\
            <methodCall>\
                <methodName>foobar</methodName>\
                <params>\
                    <param>\
                        <value><string>South Dakota</string></value>\
                    </param>\
                    <param>\
                        <value>\
                            <struct>\
                                <member>\
                                    <name>foo</name>\
                                    <value><int>42</int></value>\
                                </member>\
                                <member>\
                                    <name>bar</name>\
                                    <value><string>baz</string></value>\
                                </member>\
                            </struct>\
                        </value>\
                    </param>\
                </params>\
            </methodCall>",
        );
    }

    #[test]
    fn decodes_success_response() {
        assert_eq!(
            Response::success(vec![
                Value::String("South Dakota".into()),
                Value::Struct(vec![
                    ("foo".into(), Value::Int(42)),
                    ("bar".into(), Value::String("baz".into())),
                    ("bar2".into(), Value::String("baz2".into())),
                ]),
            ]),
            Response::read_xml(
                r#"<?xml version="1.0"?>
<methodResponse>
    <params>
        <param>
            <value><string>South Dakota</string></value>
        </param>
        <param>
            <value>
                <struct>
                    <member>
                        <name>foo</name>
                        <value><int>42</int></value>
                    </member>
                    <member>
                        <name>bar</name>
                        <value><string>baz</string></value>
                    </member>
                    <member>
                        <name>bar2</name>
                        <value>baz2</value>
                    </member>
                </struct>
            </value>
        </param>
    </params>
</methodResponse>"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn decodes_failure_response() {
        assert_eq!(
            Response::failure(4, "Too many parameters.".into()),
            Response::read_xml(
                r#"<?xml version="1.0"?>
<methodResponse>
    <fault>
        <value>
            <struct>
                <member>
                    <name>faultCode</name>
                    <value><int>4</int></value>
                </member>
                <member>
                    <name>faultString</name>
                    <value><string>Too many parameters.</string></value>
                </member>
            </struct>
        </value>
    </fault>
</methodResponse>"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn decodes_request() {
        assert_eq!(
            Request::new(
                "foobar".into(),
                vec![
                    Value::String("South Dakota".into()),
                    Value::Struct(vec![
                        ("foo".into(), Value::Int(42)),
                        ("bar".into(), Value::String("baz".into())),
                    ])
                ]
            ),
            Request::read_xml(
                r#"<?xml version="1.0"?>
<methodCall>
    <methodName>foobar</methodName>
    <params>
        <param>
            <value><string>South Dakota</string></value>
        </param>
        <param>
            <value>
                <struct>
                    <member>
                        <name>foo</name>
                        <value><i4>42</i4></value>
                    </member>
                    <member>
                        <name>bar</name>
                        <value><string>baz</string></value>
                    </member>
                </struct>
            </value>
        </param>
    </params>
</methodCall>"#
            )
            .unwrap(),
        );
    }
}
