use super::super::ser::Serializer;
use super::super::Value;
use serde::Serialize;
use serde_bytes::Bytes;
use std::collections::HashMap;

#[test]
fn writes_bool() {
    assert_eq!(true.serialize(Serializer {}).unwrap(), Value::Bool(true));
    assert_eq!(false.serialize(Serializer {}).unwrap(), Value::Bool(false));
}

#[test]
fn writes_integers_as_ints_or_strings_if_too_big() {
    assert_eq!(200u8.serialize(Serializer {}).unwrap(), Value::Int(200));
    assert_eq!(
        42_000u16.serialize(Serializer {}).unwrap(),
        Value::Int(42_000)
    );
    assert_eq!(
        4_200_000_000u32.serialize(Serializer {}).unwrap(),
        Value::String("4200000000".into())
    );
    assert_eq!(
        10_000_000_000_000_000_000u64
            .serialize(Serializer {})
            .unwrap(),
        Value::String("10000000000000000000".into())
    );
    assert_eq!(
        (-42 as i8).serialize(Serializer {}).unwrap(),
        Value::Int(-42)
    );
    assert_eq!(
        (-26_000 as i16).serialize(Serializer {}).unwrap(),
        Value::Int(-26_000)
    );
    assert_eq!(
        (-2_000_000_000 as i32).serialize(Serializer {}).unwrap(),
        Value::Int(-2000000000)
    );
    assert_eq!(
        (-8_000_000_000_000_000_000 as i64)
            .serialize(Serializer {})
            .unwrap(),
        Value::String("-8000000000000000000".into())
    );
    assert_eq!(42i8.serialize(Serializer {}).unwrap(), Value::Int(42));
    assert_eq!(
        26_000i16.serialize(Serializer {}).unwrap(),
        Value::Int(26_000)
    );
    assert_eq!(
        2_000_000_000i32.serialize(Serializer {}).unwrap(),
        Value::Int(2000000000)
    );
    assert_eq!(
        8_000_000_000_000_000_000i64
            .serialize(Serializer {})
            .unwrap(),
        Value::String("8000000000000000000".into())
    );
}

#[test]
fn writes_floats() {
    assert_eq!(
        3.25f32.serialize(Serializer {}).unwrap(),
        Value::Double(3.25f64)
    );
    assert_eq!(
        3.25f64.serialize(Serializer {}).unwrap(),
        Value::Double(3.25f64)
    );
}

#[test]
fn writes_chars_as_strings() {
    assert_eq!(
        'A'.serialize(Serializer {}).unwrap(),
        Value::String("A".into())
    );
    assert_eq!(
        ' '.serialize(Serializer {}).unwrap(),
        Value::String(" ".into())
    );
}

#[test]
fn writes_strings() {
    assert_eq!(
        "static string".serialize(Serializer {}).unwrap(),
        Value::String("static string".into())
    );
    assert_eq!(
        String::from("string object")
            .serialize(Serializer {})
            .unwrap(),
        Value::String("string object".into())
    );
}

#[test]
fn writes_bytes_as_base64() {
    assert_eq!(
        Bytes::new(b"0123").serialize(Serializer {}).unwrap(),
        Value::Base64(vec![48, 49, 50, 51])
    );
}

#[test]
fn writes_options_as_one_elem_or_empty_array() {
    let none: Option<i32> = None;
    assert_eq!(
        none.serialize(Serializer {}).unwrap(),
        Value::Array(Vec::new())
    );
    assert_eq!(
        Some(33i32).serialize(Serializer {}).unwrap(),
        Value::Array(vec![Value::Int(33)])
    );
    assert_eq!(
        Some("txt").serialize(Serializer {}).unwrap(),
        Value::Array(vec![Value::String("txt".into())])
    );
}

#[test]
fn writes_units_as_empty_struct() {
    assert_eq!(
        ().serialize(Serializer {}).unwrap(),
        Value::Struct(HashMap::new())
    );

    #[derive(Serialize)]
    struct Helper;

    assert_eq!(
        Helper.serialize(Serializer {}).unwrap(),
        Value::Struct(HashMap::new())
    );
}

#[test]
fn writes_newtype_struct_as_its_content() {
    #[derive(Serialize)]
    struct HelperInt(i32);
    #[derive(Serialize)]
    struct HelperString(String);

    assert_eq!(
        HelperInt(33).serialize(Serializer {}).unwrap(),
        Value::Int(33)
    );
    assert_eq!(
        HelperString("txt".into()).serialize(Serializer {}).unwrap(),
        Value::String("txt".into())
    );
}

#[test]
fn writes_vector_as_array() {
    assert_eq!(
        vec![33, 15, 44, 12].serialize(Serializer {}).unwrap(),
        Value::Array(vec![
            Value::Int(33),
            Value::Int(15),
            Value::Int(44),
            Value::Int(12),
        ])
    );
    assert_eq!(
        vec!['a', 'b', 'c', 'd'].serialize(Serializer {}).unwrap(),
        Value::Array(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("d".into()),
        ])
    );
}

#[test]
fn writes_tuple_as_array() {
    assert_eq!(
        (4, 1_000_000_000_000i64, "hello", true)
            .serialize(Serializer {})
            .unwrap(),
        Value::Array(vec![
            Value::Int(4),
            Value::String("1000000000000".into()),
            Value::String("hello".into()),
            Value::Bool(true),
        ])
    );

    #[derive(Serialize)]
    struct Helper(u8, u64, String, bool);

    assert_eq!(
        Helper(4, 1_000_000_000_000u64, "hello".into(), true)
            .serialize(Serializer {})
            .unwrap(),
        Value::Array(vec![
            Value::Int(4),
            Value::String("1000000000000".into()),
            Value::String("hello".into()),
            Value::Bool(true),
        ])
    );
}

#[test]
fn writes_struct_as_struct() {
    #[derive(Serialize)]
    struct Helper {
        foo: u8,
        bar: u64,
        baz: String,
        qux: bool,
    };

    let mut members = HashMap::new();
    members.insert("foo".into(), Value::Int(4));
    members.insert("bar".into(), Value::String("1000000000000".into()));
    members.insert("baz".into(), Value::String("hello".into()));
    members.insert("qux".into(), Value::Bool(true));

    assert_eq!(
        Helper {
            foo: 4,
            bar: 1_000_000_000_000u64,
            baz: "hello".into(),
            qux: true,
        }
        .serialize(Serializer {})
        .unwrap(),
        Value::Struct(members)
    );
}

#[test]
fn writes_map_as_struct() {
    let mut data = HashMap::new();
    data.insert("foo", vec![44i8, 12]);
    data.insert("bar", vec![]);
    data.insert("baz", vec![-3, 44, 28]);

    let mut members = HashMap::new();
    members.insert(
        "foo".into(),
        Value::Array(vec![Value::Int(44), Value::Int(12)]),
    );
    members.insert("bar".into(), Value::Array(vec![]));
    members.insert(
        "baz".into(),
        Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
    );

    assert_eq!(
        data.serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );
}

#[test]
fn map_accepts_string_keys() {
    let mut data = HashMap::new();
    data.insert(String::from("foo"), vec![44i8, 12]);
    data.insert(String::from("bar"), vec![]);
    data.insert(String::from("baz"), vec![-3, 44, 28]);

    let mut members = HashMap::new();
    members.insert(
        "foo".into(),
        Value::Array(vec![Value::Int(44), Value::Int(12)]),
    );
    members.insert("bar".into(), Value::Array(vec![]));
    members.insert(
        "baz".into(),
        Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
    );

    assert_eq!(
        data.serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );
}

#[test]
fn map_accepts_integer_keys() {
    let mut data = HashMap::new();
    data.insert(12, vec![44i8, 12]);
    data.insert(-33, vec![]);
    data.insert(44, vec![-3, 44, 28]);

    let mut members = HashMap::new();
    members.insert(
        "12".into(),
        Value::Array(vec![Value::Int(44), Value::Int(12)]),
    );
    members.insert("-33".into(), Value::Array(vec![]));
    members.insert(
        "44".into(),
        Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
    );

    assert_eq!(
        data.serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );
}

#[test]
fn map_accepts_char_keys() {
    let mut data = HashMap::new();
    data.insert('a', vec![44i8, 12]);
    data.insert('b', vec![]);
    data.insert('c', vec![-3, 44, 28]);

    let mut members = HashMap::new();
    members.insert(
        "a".into(),
        Value::Array(vec![Value::Int(44), Value::Int(12)]),
    );
    members.insert("b".into(), Value::Array(vec![]));
    members.insert(
        "c".into(),
        Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
    );

    assert_eq!(
        data.serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );
}

#[test]
fn map_accepts_boolean_keys() {
    let mut data = HashMap::new();
    data.insert(true, vec![44i8, 12]);
    data.insert(false, vec![]);

    let mut members = HashMap::new();
    members.insert(
        "true".into(),
        Value::Array(vec![Value::Int(44), Value::Int(12)]),
    );
    members.insert("false".into(), Value::Array(vec![]));

    assert_eq!(
        data.serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );
}

#[test]
fn rejects_maps_with_unsupported_keys() {
    let mut data = HashMap::new();
    data.insert(Some(4), vec![44i8, 12]);
    data.insert(Some(3), vec![]);
    data.insert(Some(2), vec![-3, 44, 28]);
    data.serialize(Serializer {}).unwrap_err();
}

#[test]
fn writes_variant_as_one_member_struct() {
    #[derive(Debug, Serialize)]
    enum Helper {
        Foo,
        Bar(i32),
        Baz(bool, &'static str),
        Qux { alpha: i32, beta: Vec<bool> },
    };

    let mut members = HashMap::new();
    members.insert("Foo".into(), Value::Struct(HashMap::new()));
    assert_eq!(
        Helper::Foo.serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );

    let mut members = HashMap::new();
    members.insert("Bar".into(), Value::Int(44));
    assert_eq!(
        Helper::Bar(44).serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );

    let mut members = HashMap::new();
    members.insert(
        "Baz".into(),
        Value::Array(vec![Value::Bool(false), Value::String("tsk".into())]),
    );
    assert_eq!(
        Helper::Baz(false, "tsk").serialize(Serializer {}).unwrap(),
        Value::Struct(members)
    );

    let mut submembers = HashMap::new();
    submembers.insert("alpha".into(), Value::Int(-4));
    submembers.insert(
        "beta".into(),
        Value::Array(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
        ]),
    );

    let mut members = HashMap::new();
    members.insert("Qux".into(), Value::Struct(submembers));
    assert_eq!(
        Helper::Qux {
            alpha: -4,
            beta: vec![true, false, true],
        }
        .serialize(Serializer {})
        .unwrap(),
        Value::Struct(members)
    );
}
