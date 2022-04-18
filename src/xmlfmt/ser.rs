use super::Value;
use crate::{XmlRpcError, XmlRpcResult};
use serde::{self, Serialize};
use std::collections::HashMap;

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = XmlRpcError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeVec;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeMap;

    fn serialize_bool(self, v: bool) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_i16(self, v: i16) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_i32(self, v: i32) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Int(v))
    }

    fn serialize_i64(self, v: i64) -> XmlRpcResult<Self::Ok> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_u8(self, v: u8) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_u16(self, v: u16) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_u32(self, v: u32) -> XmlRpcResult<Self::Ok> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_u64(self, v: u64) -> XmlRpcResult<Self::Ok> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_f32(self, v: f32) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Double(f64::from(v)))
    }

    fn serialize_f64(self, v: f64) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Double(v))
    }

    fn serialize_char(self, v: char) -> XmlRpcResult<Self::Ok> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> XmlRpcResult<Self::Ok> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Base64(v.into()))
    }

    fn serialize_none(self) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Array(Vec::new()))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> XmlRpcResult<Self::Ok>
    where
        T: Serialize,
    {
        Ok(Value::Array(vec![value.serialize(self)?]))
    }

    fn serialize_unit(self) -> XmlRpcResult<Self::Ok> {
        Ok(Value::Struct(HashMap::new()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> XmlRpcResult<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> XmlRpcResult<Self::Ok> {
        let mut members = HashMap::new();
        members.insert(variant.into(), self.serialize_unit()?);
        Ok(Value::Struct(members))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> XmlRpcResult<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> XmlRpcResult<Self::Ok>
    where
        T: Serialize,
    {
        let mut members = HashMap::new();
        members.insert(variant.into(), value.serialize(self)?);
        Ok(Value::Struct(members))
    }

    fn serialize_seq(self, len: Option<usize>) -> XmlRpcResult<Self::SerializeSeq> {
        self.serialize_tuple(len.unwrap_or(0))
    }

    fn serialize_tuple(self, len: usize) -> XmlRpcResult<Self::SerializeTuple> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
            variant: None,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> XmlRpcResult<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> XmlRpcResult<Self::SerializeTupleVariant> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
            variant: Some(variant.into()),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> XmlRpcResult<Self::SerializeMap> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
            variant: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> XmlRpcResult<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> XmlRpcResult<Self::SerializeStructVariant> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
            variant: Some(variant.into()),
        })
    }
}

fn to_value<T>(value: &T) -> XmlRpcResult<Value>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Vec<Value>,
    variant: Option<String>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> XmlRpcResult<Value> {
        let content = Value::Array(self.vec);
        Ok(match self.variant {
            Some(variant) => {
                let mut members = HashMap::new();
                members.insert(variant, content);
                Value::Struct(members)
            }
            None => content,
        })
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> XmlRpcResult<Value> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> XmlRpcResult<Value> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeVec {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> XmlRpcResult<Value> {
        serde::ser::SerializeSeq::end(self)
    }
}

#[doc(hidden)]
pub struct SerializeMap {
    map: HashMap<String, Value>,
    next_key: Option<String>,
    variant: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        match to_value(&key)? {
            Value::Bool(v) => self.next_key = Some(v.to_string()),
            Value::Int(v) => self.next_key = Some(v.to_string()),
            Value::Double(v) => self.next_key = Some(v.to_string()),
            Value::String(s) => self.next_key = Some(s),
            _ => {
                return Err(XmlRpcError::UnsupportedData(
                    "Key must be a bool, int, float, char or string.".into(),
                ));
            }
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> XmlRpcResult<()>
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

    fn end(self) -> XmlRpcResult<Value> {
        let content = Value::Struct(self.map);
        Ok(match self.variant {
            Some(variant) => {
                let mut members = HashMap::new();
                members.insert(variant, content);
                Value::Struct(members)
            }
            None => content,
        })
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> XmlRpcResult<Value> {
        serde::ser::SerializeMap::end(self)
    }
}

impl serde::ser::SerializeStructVariant for SerializeMap {
    type Ok = Value;
    type Error = XmlRpcError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> XmlRpcResult<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> XmlRpcResult<Value> {
        serde::ser::SerializeMap::end(self)
    }
}
