use std::borrow::Cow;
use std::iter::Peekable;

use serde::de::{self, IntoDeserializer};

use crate::error::{Error, Result};
use crate::md::{Item, Reader};
use crate::ty::Type;

pub struct Deserializer<'de> {
    reader: Peekable<Reader<'de>>,
}

impl<'de> Deserializer<'de> {
    pub fn new(text: &'de str) -> Self {
        Self {
            reader: Reader::new(text).peekable(),
        }
    }

    fn bytes<V: de::Visitor<'de>>(&mut self, text: &str, visitor: V) -> Result<V::Value> {
        visitor.visit_byte_buf(base64::decode(text)?)
    }

    fn primitive<V: de::Visitor<'de>>(
        &mut self,
        text: Cow<'de, str>,
        uri: &'de str,
        visitor: V,
    ) -> Result<V::Value> {
        match Type::from_str(uri)? {
            Type::Bool => visitor.visit_bool(text.parse()?),
            Type::I8 => visitor.visit_i8(text.parse()?),
            Type::I16 => visitor.visit_i16(text.parse()?),
            Type::I32 => visitor.visit_i32(text.parse()?),
            Type::I64 => visitor.visit_i64(text.parse()?),
            Type::I128 => visitor.visit_i128(text.parse()?),
            Type::U8 => visitor.visit_u8(text.parse()?),
            Type::U16 => visitor.visit_u16(text.parse()?),
            Type::U32 => visitor.visit_u32(text.parse()?),
            Type::U64 => visitor.visit_u64(text.parse()?),
            Type::U128 => visitor.visit_u128(text.parse()?),
            Type::F32 => visitor.visit_f32(text.parse()?),
            Type::F64 => visitor.visit_f64(text.parse()?),
            Type::Char => visitor.visit_char(text.parse()?),
            Type::String => match text {
                Cow::Borrowed(text) => visitor.visit_borrowed_str(text),
                Cow::Owned(text) => visitor.visit_string(text),
            },
            Type::Bytes => self.bytes(text.as_ref(), visitor),

            Type::None => visitor.visit_none(),

            Type::Unit => visitor.visit_unit(),

            // Can we really do nothing with the name?
            Type::UnitStruct(..) => visitor.visit_unit(),

            // This is what the example Deserializer does but I'm not sure about it
            Type::UnitVariant(_name, variant) => visitor.visit_enum(variant.into_deserializer()),

            // All of the following are non-primitive types
            Type::Some
            | Type::NewtypeStruct(_)
            | Type::NewtypeVariant(_, _)
            | Type::Seq(_)
            | Type::Tuple(_)
            | Type::TupleStruct(_, _)
            | Type::TupleVariant(_, _, _)
            | Type::Map(_)
            | Type::Struct(_, _)
            | Type::StructVariant(_, _, _) => unreachable!(),
        }
    }

    fn ordered_list<V: de::Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        let ty = match self.reader.next().ok_or(Error::UnexpectedEOF)? {
            Item::Link { uri, .. } => Type::from_str(uri)?,
            Item::PushOrderedList | Item::PushUnorderedList | Item::PopList => unreachable!(),
        };

        match ty {
            Type::Some => {
                let value = visitor.visit_some(&mut *self)?;
                assert_eq!(self.reader.next(), Some(Item::PopList));
                Ok(value)
            }

            Type::NewtypeStruct(..) => {
                let value = visitor.visit_newtype_struct(&mut *self)?;
                assert_eq!(self.reader.next(), Some(Item::PopList));
                Ok(value)
            }

            Type::NewtypeVariant(_name, variant) => {
                let value = visitor.visit_enum(VariantDeserializer {
                    deserializer: &mut *self,
                    variant,
                })?;
                assert_eq!(self.reader.next(), Some(Item::PopList));
                Ok(value)
            }

            Type::Seq(len) => visitor.visit_seq(SeqDeserializer {
                deserializer: &mut *self,
                len,
            }),

            Type::Tuple(len) | Type::TupleStruct(_, len) => visitor.visit_seq(SeqDeserializer {
                deserializer: &mut *self,
                len: Some(len),
            }),

            Type::TupleVariant(_, variant, _) => visitor.visit_enum(VariantDeserializer {
                deserializer: &mut *self,
                variant,
            }),

            Type::Bool
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::I128
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::F32
            | Type::F64
            | Type::Char
            | Type::String
            | Type::Bytes
            | Type::None
            | Type::Unit
            | Type::UnitStruct(_)
            | Type::UnitVariant(_, _)
            | Type::Map(_)
            | Type::Struct(_, _)
            | Type::StructVariant(_, _, _) => unreachable!(),
        }
    }

    fn unordered_list<V: de::Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value> {
        let ty = match self.reader.next().ok_or(Error::UnexpectedEOF)? {
            Item::Link { uri, .. } => Type::from_str(uri)?,
            Item::PushOrderedList | Item::PushUnorderedList | Item::PopList => unreachable!(),
        };

        match ty {
            Type::Map(_) | Type::Struct(_, _) => visitor.visit_map(self),

            Type::StructVariant(_, variant, _) => visitor.visit_enum(VariantDeserializer {
                deserializer: &mut *self,
                variant,
            }),

            Type::Bool
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::I128
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::F32
            | Type::F64
            | Type::Char
            | Type::String
            | Type::Bytes
            | Type::None
            | Type::Some
            | Type::Unit
            | Type::UnitStruct(_)
            | Type::UnitVariant(_, _)
            | Type::NewtypeStruct(_)
            | Type::NewtypeVariant(_, _)
            | Type::Seq(_)
            | Type::Tuple(_)
            | Type::TupleStruct(_, _)
            | Type::TupleVariant(_, _, _) => unreachable!(),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.reader.next().ok_or(Error::UnexpectedEOF)? {
            Item::PushOrderedList => self.ordered_list(visitor),

            Item::PushUnorderedList => self.unordered_list(visitor),

            Item::PopList => {
                assert_eq!(self.reader.next(), None);
                Err(Error::UnexpectedEOF)
            }

            Item::Link { text, uri } => self.primitive(text, uri, visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.reader.next() {
            Some(Item::PushOrderedList) => seed.deserialize(self).map(Some),

            Some(Item::PopList) => Ok(None),
            Some(Item::PushUnorderedList) | Some(Item::Link { .. }) | None => unreachable!(),
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let value = seed.deserialize(&mut *self)?;
        assert_eq!(self.reader.next(), Some(Item::PopList));
        Ok(value)
    }
}

struct SeqDeserializer<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    len: Option<usize>,
}

impl<'de, 'a> de::SeqAccess<'de> for SeqDeserializer<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(Item::PopList) = self.deserializer.reader.peek() {
            self.deserializer.reader.next();
            return Ok(None);
        }

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

struct VariantDeserializer<'de, 'a> {
    deserializer: &'a mut Deserializer<'de>,
    variant: &'de str,
}

impl<'de, 'a> de::EnumAccess<'de> for VariantDeserializer<'de, 'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let value: Result<_> = seed.deserialize(self.variant.into_deserializer());
        Ok((value?, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for VariantDeserializer<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        unreachable!("unit variants are handled separately")
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.deserializer)?;
        assert_eq!(self.deserializer.reader.next(), Some(Item::PopList));
        Ok(value)
    }

    fn tuple_variant<V: de::Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        use de::Deserializer;
        self.deserializer.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        use de::Deserializer;
        self.deserializer.deserialize_map(visitor)
    }
}
