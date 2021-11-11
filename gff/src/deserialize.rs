use crate::common::{
    GffFieldValue,
    UnpackStruct,
};

impl std::convert::TryFrom<&GffFieldValue> for f32 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Float(f) => Ok(*f),
            _ => Err("expected Float")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for f64 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Float(f) => Ok((*f).into()),
            GffFieldValue::Double(f) => Ok(*f),
            _ => Err("expected Float/Double")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for i8 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Char(i) => Ok(*i),
            _ => Err("expected Char")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for u8 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Byte(u) => Ok(*u),
            _ => Err("expected Byte")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for i16 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Char(i) => Ok((*i).into()),
            GffFieldValue::Short(i) => Ok(*i),
            _ => Err("expected Char/Short")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for u16 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Byte(u) => Ok((*u).into()),
            GffFieldValue::Word(u) => Ok(*u),
            _ => Err("expected Byte/Word")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for i32 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Char(i) => Ok((*i).into()),
            GffFieldValue::Short(i) => Ok((*i).into()),
            GffFieldValue::Int(i) => Ok(*i),
            _ => Err("expected Char/Short/Int")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for u32 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Byte(u) => Ok((*u).into()),
            GffFieldValue::Word(u) => Ok((*u).into()),
            GffFieldValue::DWord(u) => Ok(*u),
            _ => Err("expected Byte/Word/DWord")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for i64 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Char(i) => Ok((*i).into()),
            GffFieldValue::Short(i) => Ok((*i).into()),
            GffFieldValue::Int(i) => Ok((*i).into()),
            GffFieldValue::Int64(i) => Ok(*i),
            _ => Err("expected Char/Short/Int/Int64")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for u64 {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::Byte(u) => Ok((*u).into()),
            GffFieldValue::Word(u) => Ok((*u).into()),
            GffFieldValue::DWord(u) => Ok((*u).into()),
            GffFieldValue::DWord64(u) => Ok(*u),
            _ => Err("expected Byte/Word/DWord/DWord64")
        }
    }
}

impl std::convert::TryFrom<&GffFieldValue> for String {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::CExoString(s) => Ok(s.to_string()),
            _ => Err("expect CExoString"),
        }
    }
}

impl<T> std::convert::TryFrom<&GffFieldValue> for Vec<T> where T: UnpackStruct {
    type Error = &'static str;

    fn try_from(value: &GffFieldValue) -> Result<Self, Self::Error> {
        match value {
            GffFieldValue::List(v) => {
                v.iter()
                    .map(|x| { T::unpack(&x) })
                    .collect::<Result<Vec<T>, Self::Error>>()
            },
            _ => Err("Expected GffFieldValue::List"),
        }
    }
}
