// FIXME: we have to choose how we handel escapes cause rn it's wrong
mod error;
mod ty;

pub mod md;

pub mod ser;

pub mod de;

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_value::Value as SerdeValue;

    use super::*;

    // XXX: Could we make this exercise more of the code?
    // FIXME: is there no way to test Bytes?
    fn st_value() -> impl Strategy<Value = SerdeValue> {
        let st_leaf = prop_oneof![
            any::<bool>().prop_map(SerdeValue::Bool),
            any::<u8>().prop_map(SerdeValue::U8),
            any::<u16>().prop_map(SerdeValue::U16),
            any::<u32>().prop_map(SerdeValue::U32),
            any::<u64>().prop_map(SerdeValue::U64),
            any::<i8>().prop_map(SerdeValue::I8),
            any::<i16>().prop_map(SerdeValue::I16),
            any::<i32>().prop_map(SerdeValue::I32),
            any::<i64>().prop_map(SerdeValue::I64),
            any::<char>().prop_map(SerdeValue::Char),
            any::<String>().prop_map(SerdeValue::String),
            Just(SerdeValue::Unit),
        ];

        st_leaf.prop_recursive(8, 256, 10, |inner| {
            prop_oneof![
                prop::option::of(inner.clone())
                    .prop_map(|opt| SerdeValue::Option(opt.map(Box::new))),
                inner
                    .clone()
                    .prop_map(|value| SerdeValue::Newtype(Box::new(value))),
                prop::collection::vec(inner.clone(), 0..10).prop_map(SerdeValue::Seq),
                prop::collection::btree_map(inner.clone(), inner, 0..10).prop_map(SerdeValue::Map)
            ]
        })
    }

    fn roundtrip<T: Serialize + for<'de> Deserialize<'de>>(value: &T) -> T {
        let mut buf = Vec::new();
        value
            .serialize(&mut ser::Serializer::new(&mut buf))
            .unwrap();
        let buf = String::from_utf8(buf).unwrap();
        T::deserialize(&mut de::Deserializer::new(&buf)).unwrap()
    }

    proptest! {
        // Property: a Value is equal to itself if roundtripped
        #[test]
        fn proptest_roundtrip(value in st_value()) {
            prop_assert_eq!(roundtrip(&value), value);
        }

        // Property: two unequal values are still unequal if roundtripped
        // This avoids situations like serde_json where Some(()) and None are the same
        #[test]
        fn proptest_unequal(value1 in st_value(), value2 in st_value()) {
            prop_assume!(value1 != value2);
            prop_assert_ne!(roundtrip(&value1), roundtrip(&value2));
        }
    }
}
