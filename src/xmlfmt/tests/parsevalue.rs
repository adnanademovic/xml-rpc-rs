use super::super::*;
use serde::Deserialize;
use std::collections::HashMap;

static BAD_DATA: &'static str = "Bad data provided";

#[test]
fn reads_pod_xml_value() {
    let data = r#"<?xml version="1.0"?><string>South Dakota</string>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::String("South Dakota".into()));
    let data = r#"<?xml version="1.0"?><string />"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::String("".into()));
    let data = r#"<?xml version="1.0"?><string></string>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::String("".into()));

    let data = r#"<?xml version="1.0"?><int>-33</int>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Int(-33));
    let data = r#"<?xml version="1.0"?><i4>-33</i4>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Int(-33));

    let data = r#"<?xml version="1.0"?><boolean>1</boolean>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Bool(true));
    let data = r#"<?xml version="1.0"?><boolean>0</boolean>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Bool(false));

    let data = r#"<?xml version="1.0"?><double>-44.2</double>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Double(-44.2));

    let data = r#"<?xml version="1.0"?><dateTime.iso8601>33</dateTime.iso8601>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::DateTime("33".into()));

    let data = r#"<?xml version="1.0"?><base64>Zm9vYmFy</base64>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Base64("foobar".into()));
}

#[test]
fn reads_empty_array_xml_value() {
    let data = r#"<?xml version="1.0"?>
<array>
    <data>
    </data>
</array>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Array(vec![]));
}

#[test]
fn reads_array_xml_value() {
    let data = r#"<?xml version="1.0"?>
<array>
    <data>
        <value><i4>33</i4></value>
        <value><i4>-12</i4></value>
        <value><i4>44</i4></value>
    </data>
</array>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(
        data,
        Value::Array(vec![Value::Int(33), Value::Int(-12), Value::Int(44)])
    );
}

#[test]
fn reads_empty_struct_xml_value() {
    let data = r#"<?xml version="1.0"?><struct></struct>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Struct(HashMap::<String, Value>::new()));
}

#[test]
fn reads_tagged_and_untagged_strings() {
    let data = r#"<?xml version="1.0"?>
<array>
    <data>
        <value><string>foo</string></value>
        <value><string></string></value>
        <value><string /></value>
        <value>bar</value>
        <value></value>
        <value />
    </data>
</array>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(
        data,
        Value::Array(vec![
            Value::String("foo".into()),
            Value::String(String::new()),
            Value::String(String::new()),
            Value::String("bar".into()),
            Value::String(String::new()),
            Value::String(String::new()),
        ])
    );
}

#[test]
fn reads_struct_xml_value() {
    let mut fields = HashMap::<String, Value>::new();
    fields.insert("foo".into(), Value::Int(42));
    fields.insert("bar".into(), Value::String("baz".into()));
    let data = r#"<?xml version="1.0"?>
<struct>
    <member>
        <name>foo</name>
        <value><i4>42</i4></value>
    </member>
    <member>
        <name>bar</name>
        <value><string>baz</string></value>
    </member>
</struct>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Value::Struct(fields));
}

#[test]
fn reads_response() {
    let mut fields = HashMap::<String, Value>::new();
    fields.insert("foo".into(), Value::Int(42));
    fields.insert("bar".into(), Value::String("baz".into()));
    fields.insert("bar2".into(), Value::String("baz2".into()));
    let params = vec![Value::String("South Dakota".into()), Value::Struct(fields)];
    let data = r#"<?xml version="1.0"?>
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
                        <value><i4>42</i4></value>
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
</methodResponse>"#;
    let data = parse::response(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data, Ok(params));
}

#[test]
fn reads_fault() {
    let data = r#"<?xml version="1.0"?>
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
</methodResponse>"#;
    let data = parse::response(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(
        data,
        Err(Fault {
            code: 4,
            message: "Too many parameters.".into(),
        })
    );
}

#[test]
fn reads_call() {
    let mut fields = HashMap::<String, Value>::new();
    fields.insert("foo".into(), Value::Int(42));
    fields.insert("bar".into(), Value::String("baz".into()));
    let data = r#"<?xml version="1.0"?>
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
</methodCall>"#;
    let data = parse::call(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(data.name, String::from("foobar"));
    assert_eq!(
        data.params,
        vec![Value::String("South Dakota".into()), Value::Struct(fields)]
    );
}

#[test]
fn reads_array_structure_xml_value() {
    let data = r#"<?xml version="1.0"?>
<array>
    <data>
        <value><i4>33</i4></value>
        <value><i4>-12</i4></value>
        <value><i4>44</i4></value>
    </data>
</array>"#;
    let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
    let data = Vec::<i32>::deserialize(data).expect(BAD_DATA);
    assert_eq!(data, vec![33, -12, 44]);
}

fn ser_and_de(value: Value) {
    ser_and_de_response_value(Ok(vec![value]));
}

fn ser_and_de_call_value(value: Call) {
    use super::super::value::ToXml;
    let data = value.to_xml();
    let data = parse::call(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(value, data);
}

fn ser_and_de_response_value(value: Response) {
    use super::super::value::ToXml;
    let data = value.to_xml();
    let data = parse::response(data.as_bytes()).expect(BAD_DATA);
    assert_eq!(value, data);
}

#[test]
fn writes_pod_xml_value() {
    ser_and_de(Value::String("South Dakota".into()));
    ser_and_de(Value::String("".into()));
    ser_and_de(Value::String("".into()));
    ser_and_de(Value::Int(-33));
    ser_and_de(Value::Int(-33));
    ser_and_de(Value::Bool(true));
    ser_and_de(Value::Bool(false));
    ser_and_de(Value::Double(-44.2));
    ser_and_de(Value::DateTime("33".into()));
    ser_and_de(Value::Base64("ASDF=".into()));
}

#[test]
fn writes_array_xml_value() {
    ser_and_de(Value::Array(vec![
        Value::Int(33),
        Value::Int(-12),
        Value::Int(44),
    ]));
}

#[test]
fn writes_struct_xml_value() {
    let mut fields = HashMap::<String, Value>::new();
    fields.insert("foo".into(), Value::Int(42));
    fields.insert("bar".into(), Value::String("baz".into()));
    ser_and_de(Value::Struct(fields));
}

#[test]
fn writes_response() {
    let mut fields = HashMap::<String, Value>::new();
    fields.insert("foo".into(), Value::Int(42));
    fields.insert("bar".into(), Value::String("baz".into()));
    let params = vec![Value::String("South Dakota".into()), Value::Struct(fields)];
    ser_and_de_response_value(Ok(params))
}

#[test]
fn writes_fault() {
    ser_and_de_response_value(Err(Fault {
        code: 4,
        message: "Too many parameters.".into(),
    }));
}

#[test]
fn writes_call() {
    let mut fields = HashMap::<String, Value>::new();
    fields.insert("foo".into(), Value::Int(42));
    fields.insert("bar".into(), Value::String("baz".into()));
    ser_and_de_call_value(Call {
        name: String::from("foobar"),
        params: vec![Value::String("South Dakota".into()), Value::Struct(fields)],
    });
}

#[test]
fn reads_and_writes_empty_call() {
    ser_and_de_call_value(Call {
        name: String::new(),
        params: Vec::new(),
    })
}

#[test]
fn reads_and_writes_empty_response() {
    ser_and_de_response_value(Ok(vec![]))
}
