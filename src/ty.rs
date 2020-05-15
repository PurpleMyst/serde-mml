use std::fmt;

#[derive(Debug)]
pub enum Type {
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
    UnitStruct(&'static str),
    UnitVariant(&'static str, &'static str),
    NewtypeStruct(&'static str),
    NewtypeVariant(&'static str, &'static str),
    Seq(Option<usize>),
    Tuple(usize),
    TupleStruct(&'static str, usize),
    TupleVariant(&'static str, &'static str, usize),
    Map(Option<usize>),
    Struct(&'static str, usize),
    StructVariant(&'static str, &'static str, usize),
}

impl fmt::Display for Type {
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
            Type::Some => f.pad("serde://option/some"),
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
