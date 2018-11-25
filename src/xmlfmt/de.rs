use super::error::{Error, Result};
use super::Value;
use serde::de::{
    DeserializeSeed, EnumAccess, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::{self, Deserializer};
use std;
use std::collections::HashMap;
use std::vec;

impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Int(v) => visitor.visit_i32(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::DateTime(v) | Value::String(v) => visitor.visit_string(v),
            Value::Double(v) => visitor.visit_f64(v),
            Value::Base64(v) => visitor.visit_bytes(v.as_slice()),
            Value::Array(v) => {
                let len = v.len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = visitor.visit_seq(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"fewer elements in array",
                    ))
                }
            }
            Value::Struct(v) => {
                let len = v.len();
                let mut deserializer = MapDeserializer::new(v);
                let map = visitor.visit_map(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"fewer elements in map",
                    ))
                }
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            Value::String(v) => match v.as_str() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err(serde::de::Error::invalid_value(
                    Unexpected::Str(&v),
                    &visitor,
                )),
            },
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Double(v) => visitor.visit_f32(v as f32),
            Value::String(v) => {
                let x: Result<f32> = v
                    .parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), &visitor));
                visitor.visit_f32(x?)
            }
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Double(v) => visitor.visit_f64(v),
            Value::String(v) => {
                let x: Result<f64> = v
                    .parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), &visitor));
                visitor.visit_f64(x?)
            }
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            if v.len() != 1 {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Str(&v),
                    &"string with a single character",
                ));
            }
            visitor.visit_char(v.chars().next().unwrap())
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_str(&v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_string(v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Base64(v) = self {
            visitor.visit_bytes(v.as_slice())
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Base64(v) = self {
            visitor.visit_byte_buf(v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Array(mut v) = self {
            let v1 = v.pop();
            if !v.is_empty() {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Seq,
                    &"array with a single element",
                ));
            }
            match v1 {
                Some(x) => visitor.visit_some(x),
                None => visitor.visit_none(),
            }
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Struct(v) = self {
            if !v.is_empty() {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"empty map",
                ));
            }
            visitor.visit_unit()
        } else {
            Err(serde::de::Error::invalid_value(
                self.unexpected(),
                &"empty map",
            ))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Struct(members) => {
                let mut member_iter = members.into_iter();
                if let Some((key, value)) = member_iter.next() {
                    if member_iter.next().is_none() {
                        return visitor.visit_enum(EnumDeserializer {
                            variant: key,
                            value: value,
                        });
                    }
                }
                Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"map with a single key",
                ))
            }
            other => Err(serde::de::Error::invalid_value(
                other.unexpected(),
                &"map with a single key",
            )),
        }
    }

    forward_to_deserialize_any! {
        identifier ignored_any
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        let ret = try!(visitor.visit_seq(&mut self));
        let remaining = self.iter.len();
        if remaining == 0 {
            Ok(ret)
        } else {
            Err(serde::de::Error::invalid_length(
                len,
                &"fewer elements in array",
            ))
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <HashMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: HashMap<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct EnumDeserializer {
    variant: String,
    value: Value,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = Value;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Value)>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value;
        let variant = Value::String(self.variant);
        seed.deserialize(variant).map(|v| (v, value))
    }
}

impl<'de> VariantAccess<'de> for Value {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if let Value::Struct(v) = self {
            if !v.is_empty() {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"empty map",
                ));
            }
            Ok(())
        } else {
            Err(serde::de::Error::invalid_value(
                self.unexpected(),
                &"empty map",
            ))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct("", fields, visitor)
    }
}

trait FromI32 {
    fn from_i32(v: i32) -> Self;
}

macro_rules! impl_from_i32 {
    ($($ty:ty)*) => {
        $(
            impl FromI32 for $ty {
                #[inline]
                fn from_i32(v: i32) -> $ty {
                    v as $ty
                }
            }
        )*
    }
}

impl_from_i32!(u8 u16 u32 u64 i8 i16 i32);

impl FromI32 for i64 {
    #[inline]
    fn from_i32(v: i32) -> i64 {
        v.into()
    }
}

fn handle_integer<'de, T, V>(value: Value, visitor: &V) -> Result<T>
where
    T: FromI32 + std::str::FromStr,
    V: Visitor<'de>,
{
    match value {
        Value::Int(v) => Ok(T::from_i32(v)),
        Value::String(v) => v
            .parse()
            .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), visitor)),
        _ => Err(serde::de::Error::invalid_value(value.unexpected(), visitor)),
    }
}
