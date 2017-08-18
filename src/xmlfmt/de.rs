#[derive(Debug, PartialEq, Deserialize)]
enum XmlValue {
    #[serde(rename = "i4")]
    I4(i32),
    #[serde(rename = "int")]
    Int(i32),
    #[serde(rename = "boolean")]
    Bool(i32),
    #[serde(rename = "string")]
    Str(String),
    #[serde(rename = "double")]
    Double(String),
    #[serde(rename = "dateTime.iso8601")]
    DateTime(String),
    #[serde(rename = "base64")]
    Base64(String),
    #[serde(rename = "array")]
    Array(XmlArray),
    #[serde(rename = "struct")]
    Struct(XmlStruct),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "methodCall")]
struct XmlCall {
    #[serde(rename = "methodName")]
    pub name: String,
    pub params: XmlParams,
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponseResult {
    #[serde(rename = "params")]
    Success(XmlParams),
    #[serde(rename = "fault")]
    Failure { value: XmlValue },
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponse {
    #[serde(rename = "methodResponse")]
    Response(XmlResponseResult),
}


#[derive(Debug, PartialEq, Deserialize)]
struct XmlParams {
    #[serde(rename = "param")]
    pub params: Vec<XmlParamData>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParamData {
    pub value: XmlValue,
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArray {
    #[serde(rename = "data")]
    pub data: XmlArrayData,
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArrayData {
    pub value: Vec<XmlValue>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStruct {
    #[serde(rename = "member")]
    pub members: Vec<XmlStructItem>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStructItem {
    pub name: String,
    pub value: XmlValue,
}

#[cfg(test)]
mod tests {
    use serde_xml_rs::deserialize;
    use super::*;

    static BAD_DATA: &'static str = "Bad data provided";

    #[test]
    fn reads_pod_xml_value() {
        let data = r#"<?xml version="1.0"?><string>South Dakota</string>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Str("South Dakota".into()));
        let data = r#"<?xml version="1.0"?><string />"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Str("".into()));
        let data = r#"<?xml version="1.0"?><string></string>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Str("".into()));

        let data = r#"<?xml version="1.0"?><int>-33</int>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Int(-33));
        let data = r#"<?xml version="1.0"?><i4>-33</i4>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::I4(-33));

        let data = r#"<?xml version="1.0"?><boolean>1</boolean>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Bool(1));
        let data = r#"<?xml version="1.0"?><boolean>0</boolean>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Bool(0));

        let data = r#"<?xml version="1.0"?><double>-44.2</double>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Double("-44.2".into()));

        let data = r#"<?xml version="1.0"?><dateTime.iso8601>33</dateTime.iso8601>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::DateTime("33".into()));

        let data = r#"<?xml version="1.0"?><base64>ASDF=</base64>"#;
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, XmlValue::Base64("ASDF=".into()));
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
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            XmlValue::Array(XmlArray {
                data: XmlArrayData {
                    value: vec![XmlValue::I4(33), XmlValue::I4(-12), XmlValue::I4(44)],
                },
            })
        );
    }

    #[test]
    fn reads_struct_xml_value() {
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
        let data: XmlValue = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            XmlValue::Struct(XmlStruct {
                members: vec![
                    XmlStructItem {
                        name: "foo".into(),
                        value: XmlValue::I4(42),
                    },
                    XmlStructItem {
                        name: "bar".into(),
                        value: XmlValue::Str("baz".into()),
                    },
                ],
            })
        );
    }

    #[test]
    fn reads_response() {
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
                </struct>
            </value>
        </param>
    </params>
</methodResponse>"#;
        use serde_xml_rs::deserialize;
        let data: XmlResponse = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            XmlResponse::Response(XmlResponseResult::Success(XmlParams {
                params: vec![
                    XmlParamData { value: XmlValue::Str("South Dakota".into()) },
                    XmlParamData {
                        value: XmlValue::Struct(XmlStruct {
                            members: vec![
                                XmlStructItem {
                                    name: "foo".into(),
                                    value: XmlValue::I4(42),
                                },
                                XmlStructItem {
                                    name: "bar".into(),
                                    value: XmlValue::Str("baz".into()),
                                },
                            ],
                        }),
                    },
                ],
            }))
        );
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
        use serde_xml_rs::deserialize;
        let data: XmlResponse = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            XmlResponse::Response(XmlResponseResult::Failure {
                value: XmlValue::Struct(XmlStruct {
                    members: vec![
                        XmlStructItem {
                            name: "faultCode".into(),
                            value: XmlValue::Int(4),
                        },
                        XmlStructItem {
                            name: "faultString".into(),
                            value: XmlValue::Str("Too many parameters.".into()),
                        },
                    ],
                }),
            })
        );
    }

    #[test]
    fn reads_call() {
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
        use serde_xml_rs::deserialize;
        let data: XmlCall = deserialize(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            XmlCall {
                name: "foobar".into(),
                params: XmlParams {
                    params: vec![
                        XmlParamData { value: XmlValue::Str("South Dakota".into()) },
                        XmlParamData {
                            value: XmlValue::Struct(XmlStruct {
                                members: vec![
                                    XmlStructItem {
                                        name: "foo".into(),
                                        value: XmlValue::I4(42),
                                    },
                                    XmlStructItem {
                                        name: "bar".into(),
                                        value: XmlValue::Str("baz".into()),
                                    },
                                ],
                            }),
                        },
                    ],
                },
            }
        );
    }
}
