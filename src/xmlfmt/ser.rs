use std::collections::HashMap;
use serde::{self, Serialize};
use super::Value;
use super::error::{Error, ErrorKind};

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeFail;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeFail;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("i8".into()))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("i16".into()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(v))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("i64".into()))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("u8".into()))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("u16".into()))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("u32".into()))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("u64".into()))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("f32".into()))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Double(v))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("char".into()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Base64(v.into()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("None".into()))
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        bail!(ErrorKind::UnsupportedData("Some".into()))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Struct(HashMap::new()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        bail!(ErrorKind::UnsupportedData("Unit Variant".into()))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        bail!(ErrorKind::UnsupportedData("Newtype Struct".into()))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        bail!(ErrorKind::UnsupportedData("Newtype Variant".into()))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_tuple(len.unwrap_or(0))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeVec { vec: Vec::with_capacity(len) })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        bail!(ErrorKind::UnsupportedData("Tuple Variant".into()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        bail!(ErrorKind::UnsupportedData("Struct Variant".into()))
    }
}

fn to_value<T>(value: T) -> Result<Value, Error>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Vec<Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

#[doc(hidden)]
pub struct SerializeMap {
    map: HashMap<String, Value>,
    next_key: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        match to_value(&key)? {
            Value::String(s) => self.next_key = Some(s),
            _ => bail!(ErrorKind::UnsupportedData("Key must be a string.".into())),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Error> {
        Ok(Value::Struct(self.map))
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value, Error> {
        serde::ser::SerializeMap::end(self)
    }
}

#[doc(hidden)]
pub struct SerializeFail {}

impl serde::ser::SerializeTupleVariant for SerializeFail {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        bail!("Fail")
    }

    fn end(self) -> Result<Value, Error> {
        bail!("Fail")
    }
}


impl serde::ser::SerializeStructVariant for SerializeFail {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        bail!("Fail")
    }

    fn end(self) -> Result<Value, Error> {
        bail!("Fail")
    }
}
