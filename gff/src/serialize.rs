use crate::common::{
    GffFieldValue,
    GffStruct,
    PackStruct,
};

macro_rules! gff_try_into {
    ( $gff_type:ident, $type:ident ) => {
        impl std::convert::TryInto<GffFieldValue> for &$type {
            type Error = &'static str;

            fn try_into(self) -> Result<GffFieldValue, Self::Error> {
                Ok(GffFieldValue::$gff_type(*self))
            }
        }
    }
}

gff_try_into!(Float,   f32);
gff_try_into!(Double,  f64);
gff_try_into!(Char,    i8);
gff_try_into!(Byte,    u8);
gff_try_into!(Short,   i16);
gff_try_into!(Word,    u16);
gff_try_into!(Int,     i32);
gff_try_into!(DWord,   u32);
gff_try_into!(Int64,   i64);
gff_try_into!(DWord64, u64);

impl std::convert::TryInto<GffFieldValue> for &String {
    type Error = &'static str;

    fn try_into(self) -> Result<GffFieldValue, Self::Error> {
        Ok(GffFieldValue::CExoString(self.clone()))
    }
}

impl<T> std::convert::TryInto<GffFieldValue> for &Vec<T> where T: PackStruct {
    type Error = &'static str;

    fn try_into(self) -> Result<GffFieldValue, Self::Error> {
        let mut res: Vec<GffStruct> = vec![];

        for st in self {
            res.push(st.pack()?);
        }
        Ok(
            GffFieldValue::List(res)
        )
    }
}
