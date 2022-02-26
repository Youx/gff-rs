//! Implementation of deserialization for basic types

use crate::common::{
    GffFieldValue,
    Deserialize,
};

macro_rules! gff_try_from {
    ( $gff_type:ident, $type:ident ) => {
        impl std::convert::TryFrom<&GffFieldValue> for $type {
            type Error = &'static str;

            fn try_from(value: &GffFieldValue)
                -> Result<Self, Self::Error>
            {
                match value {
                    GffFieldValue::$gff_type(val) => Ok(*val),
                    _ => Err("expected $gff_type"),
                }
            }
        }
    }
}

gff_try_from!(Float,   f32);
gff_try_from!(Double,  f64);
gff_try_from!(Byte,    u8);
gff_try_from!(Char,    i8);
gff_try_from!(Word,    u16);
gff_try_from!(Short,   i16);
gff_try_from!(DWord,   u32);
gff_try_from!(Int,     i32);
gff_try_from!(DWord64, u64);
gff_try_from!(Int64,   i64);

impl std::convert::TryFrom<&GffFieldValue> for String {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::CExoString(s) => Ok(s.to_string()),
            _ => Err("expect CExoString"),
        }
    }
}

impl<T> std::convert::TryFrom<&GffFieldValue> for Vec<T> where T: Deserialize {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::List(v) => {
                v.iter()
                    .map(|x| { T::deserialize(&x) })
                    .collect::<Result<Vec<T>, Self::Error>>()
            },
            _ => Err("Expected GffFieldValue::List"),
        }
    }
}
