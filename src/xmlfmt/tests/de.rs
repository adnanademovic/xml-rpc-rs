use super::super::Value;
use serde::Deserialize;
use serde_bytes;
use std::collections::HashMap;

#[test]
fn reads_bool() {
    assert_eq!(true, bool::deserialize(Value::Bool(true)).unwrap());
    assert_eq!(false, bool::deserialize(Value::Bool(false)).unwrap());
}

#[test]
fn reads_integers_as_ints_or_strings_if_too_big() {
    assert_eq!(200u8, u8::deserialize(Value::Int(200)).unwrap());
    assert_eq!(42_000u16, u16::deserialize(Value::Int(42_000)).unwrap());
    assert_eq!(
        4_200_000_000u32,
        u32::deserialize(Value::String("4200000000".into())).unwrap()
    );
    assert_eq!(
        10_000_000_000_000_000_000u64,
        u64::deserialize(Value::String("10000000000000000000".into())).unwrap()
    );
    assert_eq!((-42 as i8), i8::deserialize(Value::Int(-42)).unwrap());
    assert_eq!(
        (-26_000 as i16),
        i16::deserialize(Value::Int(-26_000)).unwrap()
    );
    assert_eq!(
        (-2_000_000_000 as i32),
        i32::deserialize(Value::Int(-2000000000)).unwrap()
    );
    assert_eq!(
        (-8_000_000_000_000_000_000 as i64),
        i64::deserialize(Value::String("-8000000000000000000".into())).unwrap()
    );
    assert_eq!(42i8, i8::deserialize(Value::Int(42)).unwrap());
    assert_eq!(26_000i16, i16::deserialize(Value::Int(26_000)).unwrap());
    assert_eq!(
        2_000_000_000i32,
        i32::deserialize(Value::Int(2000000000)).unwrap()
    );
    assert_eq!(
        8_000_000_000_000_000_000i64,
        i64::deserialize(Value::String("8000000000000000000".into())).unwrap()
    );
}

#[test]
fn reads_floats() {
    assert_eq!(3.25f32, f32::deserialize(Value::Double(3.25f64)).unwrap());
    assert_eq!(3.25f64, f64::deserialize(Value::Double(3.25f64)).unwrap());
}

#[test]
fn reads_chars_as_strings() {
    assert_eq!('A', char::deserialize(Value::String("A".into())).unwrap());
    assert_eq!(' ', char::deserialize(Value::String(" ".into())).unwrap());
}

#[test]
fn reads_strings() {
    assert_eq!(
        String::from("string object"),
        String::deserialize(Value::String("string object".into())).unwrap()
    );
}

#[test]
fn reads_bytes_as_base64() {
    let data: Vec<u8> = serde_bytes::deserialize(Value::Base64(vec![48, 49, 50, 51])).unwrap();
    assert_eq!(data, vec![48, 49, 50, 51]);
}

#[test]
fn reads_options_as_one_elem_or_empty_array() {
    let none: Option<i32> = None;
    assert_eq!(none, Option::deserialize(Value::Array(Vec::new())).unwrap());
    assert_eq!(
        Some(33i32),
        Option::deserialize(Value::Array(vec![Value::Int(33)])).unwrap()
    );
    assert_eq!(
        Some(String::from("txt")),
        Option::deserialize(Value::Array(vec![Value::String("txt".into())])).unwrap()
    );
}

#[test]
fn reads_units_as_empty_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Helper;

    assert_eq!(
        Helper,
        Helper::deserialize(Value::Struct(HashMap::new())).unwrap()
    );
}

#[test]
fn reads_newtype_struct_as_its_content() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct HelperInt(i32);
    #[derive(Debug, Deserialize, PartialEq)]
    struct HelperString(String);

    assert_eq!(
        HelperInt(33),
        HelperInt::deserialize(Value::Int(33)).unwrap()
    );
    assert_eq!(
        HelperString("txt".into()),
        HelperString::deserialize(Value::String("txt".into())).unwrap()
    );
}

#[test]
fn reads_vector_as_array() {
    assert_eq!(
        vec![33, 15, 44, 12],
        Vec::<usize>::deserialize(Value::Array(vec![
            Value::Int(33),
            Value::Int(15),
            Value::Int(44),
            Value::Int(12),
        ]))
        .unwrap()
    );
    assert_eq!(
        vec!['a', 'b', 'c', 'd'],
        Vec::<char>::deserialize(Value::Array(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
            Value::String("d".into()),
        ]))
        .unwrap()
    );
}

#[test]
fn reads_tuple_as_array() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Helper(u8, u64, String, bool);

    assert_eq!(
        Helper(4, 1_000_000_000_000u64, "hello".into(), true),
        Helper::deserialize(Value::Array(vec![
            Value::Int(4),
            Value::String("1000000000000".into()),
            Value::String("hello".into()),
            Value::Bool(true),
        ]))
        .unwrap()
    );
}

#[test]
fn reads_struct_as_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
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
        },
        Helper::deserialize(Value::Struct(members)).unwrap()
    );
}

#[test]
fn reads_map_as_struct() {
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

    assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
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

    assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
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

    assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
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

    assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
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

    assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
}

#[test]
fn reads_variant_as_one_member_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Helper {
        Foo,
        Bar(i32),
        Baz(bool, String),
        Qux { alpha: i32, beta: Vec<bool> },
    };

    let mut members = HashMap::new();
    members.insert("Foo".into(), Value::Struct(HashMap::new()));
    assert_eq!(
        Helper::Foo,
        Helper::deserialize(Value::Struct(members)).unwrap()
    );

    let mut members = HashMap::new();
    members.insert("Bar".into(), Value::Int(44));
    assert_eq!(
        Helper::Bar(44),
        Helper::deserialize(Value::Struct(members)).unwrap()
    );

    let mut members = HashMap::new();
    members.insert(
        "Baz".into(),
        Value::Array(vec![Value::Bool(false), Value::String("tsk".into())]),
    );
    assert_eq!(
        Helper::Baz(false, "tsk".into()),
        Helper::deserialize(Value::Struct(members)).unwrap()
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
        },
        Helper::deserialize(Value::Struct(members)).unwrap()
    );
}
