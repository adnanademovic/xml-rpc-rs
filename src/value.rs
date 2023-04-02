use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
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
    DateTime(String),
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
    Struct(HashMap<String, Value>),
}

impl Value {
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> quick_xml::Result<()> {
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
                    .write_text_content(BytesText::new(value))
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
