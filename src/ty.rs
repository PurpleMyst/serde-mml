use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unknown type URI")]
    UnknownType,

    #[error("Unknown schema, expected \"serde://\"")]
    UnknownSchema,

    #[error("Missing the domain")]
    MissingDomain,

    #[error("Missing a path fragment")]
    MissingPathFragment,

    #[error("Int parse error: {0}")]
    IntParseError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<'a> {
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    String,
    Bytes,
    None,
    Some,
    Unit,
    UnitStruct(&'a str),
    UnitVariant(&'a str, &'a str),
    NewtypeStruct(&'a str),
    NewtypeVariant(&'a str, &'a str),
    Seq(Option<usize>),
    Tuple(usize),
    TupleStruct(&'a str, usize),
    TupleVariant(&'a str, &'a str, usize),
    Map(Option<usize>),
    Struct(&'a str, usize),
    StructVariant(&'a str, &'a str, usize),
}

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => f.pad("serde://bool"),
            Type::I8 => f.pad("serde://i8"),
            Type::I16 => f.pad("serde://i16"),
            Type::I32 => f.pad("serde://i32"),
            Type::I64 => f.pad("serde://i64"),
            Type::I128 => f.pad("serde://i128"),
            Type::U8 => f.pad("serde://u8"),
            Type::U16 => f.pad("serde://u16"),
            Type::U32 => f.pad("serde://u32"),
            Type::U64 => f.pad("serde://u64"),
            Type::U128 => f.pad("serde://u128"),
            Type::F32 => f.pad("serde://f32"),
            Type::F64 => f.pad("serde://f64"),
            Type::Char => f.pad("serde://char"),
            Type::String => f.pad("serde://string"),
            Type::Bytes => f.pad("serde://bytes"),
            Type::None => f.pad("serde://none"),
            Type::Some => f.pad("serde://some"),
            Type::Unit => f.pad("serde://unit"),
            Type::UnitStruct(name) => write!(f, "serde://unit_struct/{}", name),
            Type::UnitVariant(name, variant) => {
                write!(f, "serde://unit_variant/{}/{}", name, variant)
            }
            Type::NewtypeStruct(name) => write!(f, "serde://newtype_struct/{}", name),
            Type::NewtypeVariant(name, variant) => {
                write!(f, "serde://newtype_variant/{}/{}", name, variant)
            }
            Type::Seq(Some(len)) => write!(f, "serde://seq/{}", len),
            Type::Seq(None) => f.pad("serde://seq/"),
            Type::Tuple(len) => write!(f, "serde://tuple/{}", len),
            Type::TupleStruct(name, len) => write!(f, "serde://tuple_struct/{}/{}", name, len),
            Type::TupleVariant(name, variant, len) => {
                write!(f, "serde://tuple_variant/{}/{}/{}", name, variant, len)
            }
            Type::Map(Some(len)) => write!(f, "serde://map/{}", len),
            Type::Map(None) => f.pad("serde://map/"),
            Type::Struct(name, fields) => write!(f, "serde://struct/{}/{}", name, fields),
            Type::StructVariant(name, variant, fields) => {
                write!(f, "serde://struct_variant/{}/{}/{}", name, variant, fields)
            }
        }
    }
}

impl<'a> Type<'a> {
    pub fn from_str(s: &'a str) -> Result<Self, ParseError> {
        if !s.starts_with("serde://") {
            return Err(ParseError::UnknownSchema);
        }
        let s = &s["serde://".len()..];

        let mut parts = s.split('/');

        let domain = parts.next().ok_or(ParseError::MissingDomain)?;

        fn fragment<'a>(parts: &mut std::str::Split<'a, char>) -> Result<&'a str, ParseError> {
            parts.next().ok_or(ParseError::MissingPathFragment)
        }

        fn opt_len(parts: &mut std::str::Split<'_, char>) -> Result<Option<usize>, ParseError> {
            match parts.next() {
                Some("") | None => Ok(None),
                Some(s) => Ok(Some(s.parse()?)),
            }
        }

        Ok(match domain {
            "bool" => Type::Bool,
            "i8" => Type::I8,
            "i16" => Type::I16,
            "i32" => Type::I32,
            "i64" => Type::I64,
            "i128" => Type::I128,
            "u8" => Type::U8,
            "u16" => Type::U16,
            "u32" => Type::U32,
            "u64" => Type::U64,
            "u128" => Type::U128,
            "f32" => Type::F32,
            "f64" => Type::F64,
            "char" => Type::Char,
            "string" => Type::String,
            "bytes" => Type::Bytes,
            "none" => Type::None,
            "some" => Type::Some,
            "unit" => Type::Unit,
            "unit_struct" => Type::UnitStruct(fragment(&mut parts)?),
            "unit_variant" => Type::UnitVariant(fragment(&mut parts)?, fragment(&mut parts)?),
            "newtype_struct" => Type::NewtypeStruct(fragment(&mut parts)?),
            "newtype_variant" => Type::NewtypeVariant(fragment(&mut parts)?, fragment(&mut parts)?),
            "seq" => Type::Seq(opt_len(&mut parts)?),
            "tuple" => Type::Tuple(fragment(&mut parts)?.parse()?),
            "tuple_struct" => {
                Type::TupleStruct(fragment(&mut parts)?, fragment(&mut parts)?.parse()?)
            }
            "tuple_variant" => Type::TupleVariant(
                fragment(&mut parts)?,
                fragment(&mut parts)?,
                fragment(&mut parts)?.parse()?,
            ),
            "map" => Type::Map(opt_len(&mut parts)?),
            "struct" => Type::Struct(fragment(&mut parts)?, fragment(&mut parts)?.parse()?),
            "struct_variant" => Type::StructVariant(
                fragment(&mut parts)?,
                fragment(&mut parts)?,
                fragment(&mut parts)?.parse()?,
            ),
            _ => return Err(ParseError::UnknownType),
        })
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    macro_rules! roundtrip {
        ($name:ident: [$($param:ident in $strategy:expr),+] => $expr:expr) => {
            proptest! {
                #[test]
                fn $name($($param in $strategy),+) {
                    let ty = $expr;
                    let repr = format!("{}", ty);
                    prop_assert_eq!(Type::from_str(&repr).unwrap(), ty)
                }
            }
        };

        ($name:ident: [] => $expr:expr) => {
            #[test]
            fn $name() {
                let ty = $expr;
                assert_eq!(Type::from_str(&format!("{}", ty)).unwrap(), ty)
            }
        }
    }

    const RE: &str = "[^/]+";
    roundtrip! { test_bool: [] => Type::Bool }
    roundtrip! { test_i8: [] => Type::I8 }
    roundtrip! { test_i16: [] => Type::I16 }
    roundtrip! { test_i32: [] => Type::I32 }
    roundtrip! { test_i64: [] => Type::I64 }
    roundtrip! { test_i128: [] => Type::I128 }
    roundtrip! { test_u8: [] => Type::U8 }
    roundtrip! { test_u16: [] => Type::U16 }
    roundtrip! { test_u32: [] => Type::U32 }
    roundtrip! { test_u64: [] => Type::U64 }
    roundtrip! { test_u128: [] => Type::U128 }
    roundtrip! { test_f32: [] => Type::F32 }
    roundtrip! { test_f64: [] => Type::F64 }
    roundtrip! { test_char: [] => Type::Char }
    roundtrip! { test_string: [] => Type::String }
    roundtrip! { test_bytes: [] => Type::Bytes }
    roundtrip! { test_none: [] => Type::None }
    roundtrip! { test_some: [] => Type::Some }
    roundtrip! { test_unit: [] => Type::Unit }
    roundtrip! { test_unit_struct: [name in RE] => Type::UnitStruct(&name) }
    roundtrip! { test_unit_variant: [name in RE, variant in RE] => Type::UnitVariant(&name, &variant) }
    roundtrip! { test_newtype_struct: [name in RE] => Type::NewtypeStruct(&name) }
    roundtrip! { test_newtype_variant: [name in RE, variant in RE] => Type::NewtypeVariant(&name, &variant) }
    roundtrip! { test_seq: [len in prop::option::of(any::<usize>())] => Type::Seq(len) }
    roundtrip! { test_tuple: [len in any::<usize>()] => Type::Tuple(len) }
    roundtrip! { test_tuple_struct: [name in RE, len in any::<usize>()] => Type::TupleStruct(&name, len) }
    roundtrip! { test_tuple_variant: [name in RE, variant in RE, len in any::<usize>()] => Type::TupleVariant(&name, &variant, len) }
    roundtrip! { test_map: [len in prop::option::of(any::<usize>())] => Type::Map(len) }
    roundtrip! { test_struct: [name in RE, fields in any::<usize>()] => Type::Struct(&name, fields) }
    roundtrip! { test_struct_variant: [name in RE, variant in RE, fields in any::<usize>()] => Type::StructVariant(&name, &variant, fields) }
}
