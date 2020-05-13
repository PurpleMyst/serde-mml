use std::fmt;
use std::io::{self, prelude::*};

use serde::ser;

use crate::md::{List, Writer};

pub struct Serializer<W: Write> {
    writer: Writer<W>,
    list: Option<List>,
}

pub struct SublistSerializer<'a, W: Write> {
    serializer: &'a mut Serializer<W>,
    parent: Option<List>,
}

pub struct MapSerializer<'a, W: Write> {
    serializer: &'a mut Serializer<W>,
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

    fn ser_link<Text, Uri>(&mut self, text: Text, uri: Uri) -> Result<(), Error>
    where
        Text: fmt::Display,
        Uri: fmt::Display,
    {
        self.writer.link(self.list.as_mut(), text, uri)?;
        Ok(())
    }

    fn ser_ordered_list<Text, Uri, Value>(
        &mut self,
        text: Text,
        uri: Uri,
        value: &Value,
    ) -> Result<(), Error>
    where
        Text: fmt::Display,
        Uri: fmt::Display,
        Value: ?Sized + ser::Serialize,
    {
        let mut parent = self.list.take();
        let sublist = self.writer.ordered_list(parent.as_mut())?;
        self.list = Some(sublist);
        self.ser_link(text, uri)?;
        value.serialize(&mut *self)?;
        self.list = parent;
        Ok(())
    }

    fn ser_seq<'a, Text, Uri>(
        &'a mut self,
        text: Text,
        uri: Uri,
    ) -> Result<SublistSerializer<'a, W>, Error>
    where
        Text: fmt::Display,
        Uri: fmt::Display,
    {
        let mut parent = self.list.take();
        let sublist = self.writer.ordered_list(parent.as_mut())?;
        self.list = Some(sublist);
        self.ser_link(text, uri)?;
        Ok(SublistSerializer {
            serializer: self,
            parent,
        })
    }

    fn ser_map<'a, Text, Uri>(
        &'a mut self,
        text: Text,
        uri: Uri,
    ) -> Result<MapSerializer<'a, W>, Error>
    where
        Text: fmt::Display,
        Uri: fmt::Display,
    {
        let mut parent = self.list.take();
        let sublist = self.writer.unordered_list(parent.as_mut())?;
        self.list = Some(sublist);
        self.ser_link(text, uri)?;
        Ok(MapSerializer {
            serializer: self,
            parent,
            map: None,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    CustomSerializeError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(error) => error.fmt(f),
            Error::CustomSerializeError(msg) => msg.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::CustomSerializeError(msg.to_string())
    }
}

macro_rules! serialize_int {
    ($($name:ident: $ty:ty,)*) => {
        $(
        fn $name(self, num: $ty) -> Result<Self::Ok, Self::Error> {
            self.ser_link(num, concat!("serde://", stringify!($ty)))
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
        self.writer
            .link(self.list.as_mut(), format_args!("{}", v), "serde://bool")?;
        Ok(())
    }

    serialize_int! {
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_f32: f32,
        serialize_f64: f64,
    }

    fn serialize_char(self, ch: char) -> Result<Self::Ok, Self::Error> {
        self.ser_link(ch, "serde://char")
    }

    fn serialize_str(self, s: &str) -> Result<Self::Ok, Self::Error> {
        self.ser_link(s, "serde://string")
    }

    fn serialize_bytes(self, buf: &[u8]) -> Result<Self::Ok, Self::Error> {
        // not worth it to make a ser_bytes_link
        self.writer
            .bytes_link(self.list.as_mut(), buf, "serde://bytes")?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.ser_link("None", "serde://option/none")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser_ordered_list("Some", "serde://option/some", value)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.ser_link("()", "serde://unit")
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.ser_link("name", format_args!("serde://unit_struct/{}", name))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.ser_link(
            format_args!("{}::{}", name, variant),
            format_args!("serde://unit_variant/{}/{}", name, variant),
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
        self.ser_ordered_list(name, format_args!("serde://newtype_struct/{}", name), value)
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
        self.ser_ordered_list(
            format_args!("{}::{}", name, variant),
            format_args!("serde://newtype_variant/{}/{}", name, variant),
            value,
        )
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        match len {
            Some(len) => self.ser_seq(
                format_args!("Seq of length {}", len),
                format_args!("serde://seq/{}", len),
            ),
            None => self.ser_seq(
                format_args!("Seq of unknown length"),
                format_args!("serde://seq/"),
            ),
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.ser_seq(
            format_args!("Tuple of length {}", len),
            format_args!("serde://tuple/{}", len),
        )
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.ser_seq(
            format_args!("Tuple struct {} of length {}", name, len),
            format_args!("serde://tuple_struct/{}/{}", name, len),
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
            format_args!("serde://tuple_variant/{}/{}/{}", name, variant, len),
        )
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if let Some(len) = len {
            self.ser_map(
                format_args!("Map of length {}", len),
                format_args!("serde://map/{}", len),
            )
        } else {
            self.ser_map("Map of unknown length", "serde://map")
        }
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.ser_map(
            format_args!("Struct {} of length {}", name, len),
            format_args!("serde://struct/{}/{}", name, len),
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
            format_args!("serde://struct_variant/{}/{}/{}", name, variant, len),
        )
    }

    fn collect_str<T: ?Sized>(self, s: &T) -> Result<Self::Ok, Self::Error>
    where
        T: fmt::Display,
    {
        self.ser_link(s, "serde://string")
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
