use crate::util::literal_text_in_node;
use anyhow::bail;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, FixedOffset};
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use roxmltree::Node;
use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Four-byte signed integer.
    ///
    /// Tag: `<i4>`, `<int>`
    Int(i32),
    /// Boolean with either 0 (false) or 1 (true).
    ///
    /// Tag: `<boolean>`
    Bool(bool),
    /// String value.
    ///
    /// Tag: `<string>`, or no type indicated in the tag
    String(String),
    /// Double-precision signed floating point number.
    ///
    /// Tag: `<double>`
    Double(f64),
    /// Date/time in the ISO 8601 format.
    ///
    /// Tag: `<dateTime.iso8601>`
    DateTime(DateTime<FixedOffset>),
    /// Base64-encoded binary.
    ///
    /// Tag: `<base64>`
    Base64(Vec<u8>),
    /// Array of multiple values.
    ///
    /// Tag: `<array>`
    /// An `<array>` contains a single `<data>` element, which can contain any number of `<value>`s.
    ///
    /// # Example
    ///
    /// ```xml
    /// <array>
    ///     <data>
    ///         <value><i4>12</i4></value>
    ///         <value><string>Egypt</string></value>
    ///         <value><boolean>0</boolean></value>
    ///         <value><i4>-31</i4></value>
    ///     </data>
    /// </array>
    /// ```
    Array(Vec<Value>),
    /// Map with string keys.
    ///
    /// Tag: `<struct>`
    /// A `<struct>` contains `<member>`s and each `<member>` contains a `<name>` and a `<value>`.
    ///
    /// # Example
    ///
    /// ```xml
    /// <struct>
    ///     <member>
    ///         <name>lowerBound</name>
    ///         <value><i4>18</i4></value>
    ///     </member>
    ///     <member>
    ///         <name>upperBound</name>
    ///         <value><i4>139</i4></value>
    ///     </member>
    /// </struct>
    /// ```
    Struct(Vec<(String, Value)>),
}

impl Value {
    pub(crate) fn read_xml(node: Node) -> anyhow::Result<Self> {
        if !node.has_tag_name("value") {
            bail!(
                "Expected node with tag name \"value\", found \"{}\"",
                node.tag_name().name()
            );
        }
        if let Some(child) = node.children().find(|child| child.is_element()) {
            return Ok(match child.tag_name().name() {
                "i4" | "int" => Value::Int(literal_text_in_node(child).parse()?),
                "boolean" => Value::Bool(literal_text_in_node(child).parse::<u8>()? != 0),
                "string" => Value::String(literal_text_in_node(child).to_owned()),
                "double" => Value::Double(literal_text_in_node(child).parse()?),
                "dateTime.iso8601" => {
                    Value::DateTime(DateTime::parse_from_rfc3339(literal_text_in_node(child))?)
                }
                "base64" => Value::Base64(STANDARD.decode(literal_text_in_node(child))?),
                "array" => Value::Array(
                    child
                        .children()
                        .find(|node| node.has_tag_name("data"))
                        .map_or_else(Vec::new, |data_node| {
                            data_node
                                .children()
                                .filter_map(|value_node| Value::read_xml(value_node).ok())
                                .collect()
                        }),
                ),
                "struct" => Value::Struct(
                    child
                        .children()
                        .filter(|member_node| member_node.has_tag_name("member"))
                        .filter_map(|member_node| {
                            Some((
                                literal_text_in_node(
                                    member_node
                                        .children()
                                        .find(|node| node.has_tag_name("name"))?,
                                )
                                .into(),
                                member_node
                                    .children()
                                    .find_map(|node| Value::read_xml(node).ok())?,
                            ))
                        })
                        .collect(),
                ),
                _ => bail!("Invalid value child tag {}", child.tag_name().name()),
            });
        }
        if let Some(child) = node.children().find(|child| child.is_text()) {
            return Ok(Value::String(child.text().unwrap_or("").into()));
        }
        bail!("No valid child element found");
    }

    pub(crate) fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
        writer
            .create_element("value")
            .write_inner_content(|writer| match self {
                Value::Int(value) => writer
                    .create_element("i4")
                    .write_text_content(BytesText::new(&value.to_string()))
                    .map(|_| ()),
                Value::Bool(value) => writer
                    .create_element("boolean")
                    .write_text_content(BytesText::new(if *value { "1" } else { "0" }))
                    .map(|_| ()),
                Value::String(value) => writer
                    .create_element("string")
                    .write_text_content(BytesText::new(value))
                    .map(|_| ()),
                Value::Double(value) => writer
                    .create_element("double")
                    .write_text_content(BytesText::new(&value.to_string()))
                    .map(|_| ()),
                Value::DateTime(value) => writer
                    .create_element("dateTime.iso8601")
                    .write_text_content(BytesText::new(&value.to_rfc3339()))
                    .map(|_| ()),
                Value::Base64(value) => writer
                    .create_element("base64")
                    .write_text_content(BytesText::new(&STANDARD.encode(value)))
                    .map(|_| ()),
                Value::Array(value) => writer
                    .create_element("array")
                    .write_inner_content(|writer| {
                        writer
                            .create_element("data")
                            .write_inner_content(|writer| {
                                for item in value {
                                    item.write_xml(writer)?;
                                }
                                Ok(())
                            })
                            .map(|_| ())
                    })
                    .map(|_| ()),
                Value::Struct(value) => writer
                    .create_element("struct")
                    .write_inner_content(|writer| {
                        for (key, value) in value {
                            writer
                                .create_element("member")
                                .write_inner_content(|writer| {
                                    writer
                                        .create_element("name")
                                        .write_text_content(BytesText::new(key))?;
                                    value.write_xml(writer)
                                })?;
                        }
                        Ok(())
                    })
                    .map(|_| ()),
            })
            .map(|_| ())
    }

    pub fn into_i32(self) -> Option<i32> {
        if let Value::Int(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_bool(self) -> Option<bool> {
        if let Value::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<String> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_f64(self) -> Option<f64> {
        if let Value::Double(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_date_time(self) -> Option<DateTime<FixedOffset>> {
        if let Value::DateTime(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_bytes(self) -> Option<Vec<u8>> {
        if let Value::Base64(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_array(self) -> Option<Vec<Value>> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_struct_kv_pairs(self) -> Option<Vec<(String, Value)>> {
        if let Value::Struct(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_struct(self) -> Option<HashMap<String, Value>> {
        Some(self.into_struct_kv_pairs()?.into_iter().collect())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Double(value)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        value.into_iter().collect()
    }
}

impl<T: Into<Value>> From<Vec<(String, T)>> for Value {
    fn from(value: Vec<(String, T)>) -> Self {
        value.into_iter().collect()
    }
}

impl<T: Into<Value>> From<HashMap<String, T>> for Value {
    fn from(value: HashMap<String, T>) -> Self {
        value.into_iter().collect()
    }
}

impl<T: Into<Value>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>> FromIterator<(String, T)> for Value {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        Value::Struct(iter.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;
    use std::io::Cursor;

    fn write_string(value: Value) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        value.write_xml(&mut writer).unwrap();
        String::from_utf8(writer.into_inner().into_inner()).unwrap()
    }

    #[test]
    fn integer_encoding() {
        assert_eq!("<value><i4>12</i4></value>", write_string(Value::Int(12)));

        assert_eq!("<value><i4>-33</i4></value>", write_string(Value::Int(-33)));
    }

    #[test]
    fn bool_encoding() {
        assert_eq!(
            "<value><boolean>1</boolean></value>",
            write_string(Value::Bool(true))
        );
        assert_eq!(
            "<value><boolean>0</boolean></value>",
            write_string(Value::Bool(false))
        );
    }

    #[test]
    fn string_encoding() {
        assert_eq!(
            "<value><string>abcd</string></value>",
            write_string(Value::String("abcd".into()))
        );

        assert_eq!(
            "<value><string>abcd&lt;3</string></value>",
            write_string(Value::String("abcd<3".into()))
        );

        assert_eq!(
            "<value><string></string></value>",
            write_string(Value::String("".into()))
        );
    }

    #[test]
    fn double_encoding() {
        assert_eq!(
            "<value><double>2.5</double></value>",
            write_string(Value::Double(2.5))
        );

        assert_eq!(
            "<value><double>-3000000000000</double></value>",
            write_string(Value::Double(-3_000_000_000_000.0))
        );

        assert_eq!(
            "<value><double>inf</double></value>",
            write_string(Value::Double(f64::INFINITY))
        );

        assert_eq!(
            "<value><double>NaN</double></value>",
            write_string(Value::Double(f64::NAN))
        );
    }

    #[test]
    fn datetime_encoding() {
        assert_eq!(
            "<value><dateTime.iso8601>1996-12-19T16:39:57-08:00</dateTime.iso8601></value>",
            write_string(Value::DateTime(
                DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap()
            ))
        );
    }

    #[test]
    fn base64_encoding() {
        assert_eq!(
            "<value><base64>eW91IGNhbid0IHJlYWQgdGhpcyE=</base64></value>",
            write_string(Value::Base64(b"you can't read this!".to_vec()))
        );
    }

    #[test]
    fn array_encoding() {
        assert_eq!(
            "<value><array><data></data></array></value>",
            write_string(Value::Array(vec![]))
        );

        assert_eq!(
            "<value><array><data>\
                <value><i4>5</i4></value>\
                <value><string>foo</string></value>\
                </data></array></value>",
            write_string(Value::Array(vec![
                Value::Int(5),
                Value::String("foo".into()),
            ]))
        );
    }

    #[test]
    fn struct_encoding() {
        assert_eq!(
            "<value><struct></struct></value>",
            write_string(Value::Struct(vec![]))
        );

        assert_eq!(
            "<value><struct>\
                <member>\
                    <name>foo</name>\
                    <value><i4>5</i4></value>\
                </member>\
                <member>\
                    <name>foo&lt;3</name>\
                    <value><string>foo</string></value>\
                </member>\
                </struct></value>",
            write_string(Value::Struct(vec![
                ("foo".into(), Value::Int(5)),
                ("foo<3".into(), Value::String("foo".into())),
            ]))
        );
    }

    fn read_string(value: &str) -> Value {
        let document = Document::parse(value).unwrap();
        Value::read_xml(document.root().first_child().unwrap()).unwrap()
    }

    #[test]
    fn integer_decoding() {
        assert_eq!(read_string("<value><i4>12</i4></value>"), Value::Int(12));

        assert_eq!(read_string("<value><i4>-33</i4></value>"), Value::Int(-33));
    }

    #[test]
    fn bool_decoding() {
        assert_eq!(
            read_string("<value><boolean>1</boolean></value>"),
            Value::Bool(true)
        );
        assert_eq!(
            read_string("<value><boolean>0</boolean></value>"),
            Value::Bool(false)
        );
    }

    #[test]
    fn string_decoding() {
        assert_eq!(
            read_string("<value><string>abcd</string></value>"),
            Value::String("abcd".into())
        );

        assert_eq!(
            read_string("<value><string>abcd&lt;3</string></value>"),
            Value::String("abcd<3".into())
        );

        assert_eq!(
            read_string("<value><string></string></value>"),
            Value::String("".into())
        );
    }

    #[test]
    fn double_decoding() {
        assert_eq!(
            read_string("<value><double>2.5</double></value>"),
            Value::Double(2.5)
        );

        assert_eq!(
            read_string("<value><double>-3000000000000</double></value>"),
            Value::Double(-3_000_000_000_000.0)
        );

        assert_eq!(
            read_string("<value><double>inf</double></value>"),
            Value::Double(f64::INFINITY)
        );

        assert!(read_string("<value><double>NaN</double></value>")
            .into_f64()
            .unwrap()
            .is_nan());
    }

    #[test]
    fn datetime_decoding() {
        assert_eq!(
            read_string(
                "<value><dateTime.iso8601>1996-12-19T16:39:57-08:00</dateTime.iso8601></value>"
            ),
            Value::DateTime(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap())
        );
    }

    #[test]
    fn base64_decoding() {
        assert_eq!(
            read_string("<value><base64>eW91IGNhbid0IHJlYWQgdGhpcyE=</base64></value>"),
            Value::Base64(b"you can't read this!".to_vec())
        );
    }

    #[test]
    fn array_decoding() {
        assert_eq!(
            read_string("<value><array><data></data></array></value>"),
            Value::Array(vec![])
        );

        assert_eq!(
            read_string(
                "<value><array><data>\
                <value><i4>5</i4></value>\
                <value><string>foo</string></value>\
                </data></array></value>"
            ),
            Value::Array(vec![Value::Int(5), Value::String("foo".into()),])
        );
    }

    #[test]
    fn struct_decoding() {
        assert_eq!(
            read_string("<value><struct></struct></value>"),
            Value::Struct(vec![])
        );

        assert_eq!(
            read_string(
                "<value><struct>\
                <member>\
                    <name>foo</name>\
                    <value><i4>5</i4></value>\
                </member>\
                <member>\
                    <name>foo&lt;3</name>\
                    <value><string>foo</string></value>\
                </member>\
                </struct></value>"
            ),
            Value::Struct(vec![
                ("foo".into(), Value::Int(5)),
                ("foo<3".into(), Value::String("foo".into())),
            ])
        );
    }

    #[test]
    fn tolerates_whitespace_inside_value_tag() {
        assert_eq!(read_string("<value>   <i4>12</i4></value>"), Value::Int(12));
    }
}
