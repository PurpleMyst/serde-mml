use std::fmt;
use std::io::prelude::*;

use serde::ser;

use crate::error::Error;
use crate::md::{List, Writer};
use crate::ty::Type;

pub struct Serializer<W: Write> {
    writer: Writer<W>,
    list: Option<List>,
}

pub struct SublistSerializer<'ser, W: Write> {
    serializer: &'ser mut Serializer<W>,
    parent: Option<List>,
}

pub struct MapSerializer<'ser, W: Write> {
    serializer: &'ser mut Serializer<W>,
    parent: Option<List>,
    map: Option<List>,
}

impl<W: Write> Serializer<W> {
    pub fn new(output: W) -> Self {
        Self {
            writer: Writer::new(output),
            list: None,
        }
    }

    fn ser_primitive<Value>(&mut self, value: Value, ty: Type) -> Result<(), Error>
    where
        Value: fmt::Display,
    {
        self.writer.link(self.list.as_mut(), value, ty)?;
        Ok(())
    }

    fn ser_newtype<TypeName, Value>(
        &mut self,
        ty_name: TypeName,
        ty: Type,
        value: &Value,
    ) -> Result<(), Error>
    where
        TypeName: fmt::Display,
        Value: ?Sized + ser::Serialize,
    {
        let mut parent = self.list.take();
        let sublist = self.writer.ordered_list(parent.as_mut())?;
        self.list = Some(sublist);
        self.ser_primitive(ty_name, ty)?;
        value.serialize(&mut *self)?;
        self.list = parent;
        Ok(())
    }

    fn ser_seq<'ser, SeqName>(
        &'ser mut self,
        seq_name: SeqName,
        ty: Type,
    ) -> Result<SublistSerializer<'ser, W>, Error>
    where
        SeqName: fmt::Display,
    {
        let mut parent = self.list.take();
        let sublist = self.writer.ordered_list(parent.as_mut())?;
        self.list = Some(sublist);
        self.ser_primitive(seq_name, ty)?;
        Ok(SublistSerializer {
            serializer: self,
            parent,
        })
    }

    fn ser_map<'ser, MapName>(
        &'ser mut self,
        map_name: MapName,
        ty: Type,
    ) -> Result<MapSerializer<'ser, W>, Error>
    where
        MapName: fmt::Display,
    {
        let mut parent = self.list.take();
        let sublist = self.writer.unordered_list(parent.as_mut())?;
        self.list = Some(sublist);
        self.ser_primitive(map_name, ty)?;
        Ok(MapSerializer {
            serializer: self,
            parent,
            map: None,
        })
    }
}

macro_rules! serialize_int {
    ($($name:ident: $ty:ty => $enum_ty:expr,)*) => {
        $(
        fn $name(self, num: $ty) -> Result<Self::Ok, Self::Error> {
            self.ser_primitive(num, $enum_ty)
        }
        )*
    };
}

impl<'ser, W: Write> ser::Serializer for &'ser mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SublistSerializer<'ser, W>;
    type SerializeTuple = SublistSerializer<'ser, W>;
    type SerializeTupleStruct = SublistSerializer<'ser, W>;
    type SerializeTupleVariant = SublistSerializer<'ser, W>;
    type SerializeMap = MapSerializer<'ser, W>;
    type SerializeStruct = MapSerializer<'ser, W>;
    type SerializeStructVariant = MapSerializer<'ser, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive(v, Type::Bool)
    }

    serialize_int! {
        serialize_i8: i8 => Type::I8,
        serialize_i16: i16 => Type::I16,
        serialize_i32: i32 => Type::I32,
        serialize_i64: i64 => Type::I64,
        serialize_u8: u8 => Type::U8,
        serialize_u16: u16 => Type::U16,
        serialize_u32: u32 => Type::U32,
        serialize_u64: u64 => Type::U64,
        serialize_f32: f32 => Type::F32,
        serialize_f64: f64 => Type::F64,
    }

    serde::serde_if_integer128! {
        serialize_int! {
            serialize_i128: i128 => Type::I128,
            serialize_u128: u128 => Type::U128,
        }
    }

    fn serialize_char(self, ch: char) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive(ch, Type::Char)
    }

    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive(s, Type::String)
    }

    fn serialize_bytes(self, buf: &[u8]) -> Result<Self::Ok, Self::Error> {
        // not worth it to make a ser_bytes_link
        self.writer
            .bytes_link(self.list.as_mut(), buf, Type::Bytes)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive("None", Type::None)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser_newtype("Some", Type::Some, value)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive("()", Type::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive("name", Type::UnitStruct(name))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.ser_primitive(
            format_args!("{}::{}", name, variant),
            Type::UnitVariant(name, variant),
        )
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser_newtype(name, Type::NewtypeStruct(name), value)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser_newtype(
            format_args!("{}::{}", name, variant),
            Type::NewtypeVariant(name, variant),
            value,
        )
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        match len {
            Some(len) => self.ser_seq(format_args!("Seq of length {}", len), Type::Seq(Some(len))),
            None => self.ser_seq(format_args!("Seq of unknown length"), Type::Seq(None)),
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.ser_seq(format_args!("Tuple of length {}", len), Type::Tuple(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.ser_seq(
            format_args!("Tuple struct {} of length {}", name, len),
            Type::TupleStruct(name, len),
        )
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.ser_seq(
            format_args!("Tuple variant {}::{} of length {}", name, variant, len),
            Type::TupleVariant(name, variant, len),
        )
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        match len {
            Some(len) => self.ser_map(format_args!("Map of length {}", len), Type::Map(Some(len))),
            None => self.ser_map("Map of unknown length", Type::Map(None)),
        }
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.ser_map(
            format_args!("Struct {} of length {}", name, len),
            Type::Struct(name, len),
        )
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.ser_map(
            format_args!("Struct variant {}::{} of length {}", name, variant, len),
            Type::StructVariant(name, variant, len),
        )
    }

    fn collect_str<T: ?Sized>(self, s: &T) -> Result<Self::Ok, Self::Error>
    where
        T: fmt::Display,
    {
        self.ser_primitive(s, Type::String)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'ser, W: Write> ser::SerializeSeq for SublistSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.list = self.parent;
        Ok(())
    }
}

impl<'ser, W: Write> ser::SerializeTuple for SublistSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

impl<'ser, W: Write> ser::SerializeTupleStruct for SublistSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

impl<'ser, W: Write> ser::SerializeTupleVariant for SublistSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

impl<'ser, W: Write> ser::SerializeMap for MapSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let pair = self
            .serializer
            .writer
            .ordered_list(self.serializer.list.as_mut())?;

        self.map = self.serializer.list.replace(pair);

        key.serialize(&mut *self.serializer)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.serializer)?;
        self.serializer.list = self.map.take();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.list = self.parent;
        Ok(())
    }
}

impl<'ser, W: Write> ser::SerializeStruct for MapSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as ser::SerializeMap>::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}

impl<'ser, W: Write> ser::SerializeStructVariant for MapSerializer<'ser, W> {
    type Ok = <&'ser mut Serializer<W> as ser::Serializer>::Ok;
    type Error = <&'ser mut Serializer<W> as ser::Serializer>::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as ser::SerializeMap>::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}
