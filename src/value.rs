use base64::display::Base64Display;
use base64::engine::general_purpose::STANDARD;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use xml::escape::escape_str_pcdata;

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
    /// Tag: `<i4>` or `<int>`
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

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(v) => write!(f, "<value><i4>{}</i4></value>", v),
            Value::Bool(v) => write!(
                f,
                "<value><boolean>{}</boolean></value>",
                if *v { 1 } else { 0 }
            ),
            Value::String(v) => {
                write!(
                    f,
                    "<value><string>{}</string></value>",
                    escape_str_pcdata(v)
                )
            }
            Value::Double(v) => write!(f, "<value><double>{}</double></value>", v),
            Value::DateTime(v) => {
                write!(
                    f,
                    "<value><dateTime.iso8601>{}</dateTime.iso8601></value>",
                    escape_str_pcdata(v)
                )
            }
            Value::Base64(v) => {
                write!(
                    f,
                    "<value><base64>{}</base64></value>",
                    Base64Display::new(v, &STANDARD),
                )
            }
            Value::Array(v) => {
                write!(f, "<value><array><data>",)?;
                for item in v {
                    item.fmt(f)?;
                }
                write!(f, "</data></array></value>",)?;
                Ok(())
            }
            Value::Struct(v) => {
                write!(f, "<value><struct>")?;
                for (key, value) in v {
                    write!(
                        f,
                        "<member><name>{}</name>{}</member>",
                        escape_str_pcdata(key),
                        value,
                    )?;
                }
                write!(f, "</struct></value>")?;
                Ok(())
            }
        }
    }
}
