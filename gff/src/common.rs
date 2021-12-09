extern crate num_enum;

use encoding_rs::{
    WINDOWS_1252,  // 1252
    WINDOWS_1250,  // 1250
    EUC_KR,        // 949
    BIG5,          // 950
    GBK,           // 936
    SHIFT_JIS,     // 932
};

use std::collections::HashMap;

/* {{{ GFF header */

#[derive(Debug)]
/** Tuple containing an offset, and a count */
pub struct OffsetCount (pub u32, pub u32);

impl OffsetCount {
    pub fn new() -> Self {
        OffsetCount(0, 0)
    }
}

#[derive(Debug)]
pub struct GffHeader {
    pub gff_type: [u8; 4],
    pub version: [u8; 4],
    pub structs: OffsetCount,
    pub fields: OffsetCount,
    pub labels: OffsetCount,
    pub field_data: OffsetCount,
    pub field_indices: OffsetCount,
    pub list_indices: OffsetCount,
}

impl GffHeader {
    pub fn new() -> Self {
        GffHeader {
            gff_type: *b"    ",
            version: *b"V3.2",
            structs: OffsetCount::new(),
            fields: OffsetCount::new(),
            labels: OffsetCount::new(),
            field_data: OffsetCount::new(),
            field_indices: OffsetCount::new(),
            list_indices: OffsetCount::new(),
        }
    }
}
/* }}} */
/* {{{ Gff Structs and fields */

#[derive(Debug, std::cmp::Eq, PartialEq,
    std::hash::Hash, num_enum::TryFromPrimitive,
    Copy, Clone)]
#[repr(u32)]
pub enum GffLang {
    English      = 0,
    French       = 1,
    German       = 2,
    Italian      = 3,
    Spanish      = 4,
    Polish       = 5 ,
    Korean       = 128,
    ChineseTrad  = 129,
    ChineseSimpl = 130,
    Japanese     = 131,
}

#[derive(Debug, std::cmp::Eq, PartialEq,
    std::hash::Hash, num_enum::TryFromPrimitive,
    Copy, Clone)]
#[repr(u8)]
pub enum GffGender {
    Male = 0,
    Female = 1,
}

#[derive(Debug, PartialEq)]
pub enum GffFieldValue {
    Byte(u8),
    CExoLocString(u32, HashMap<(GffLang, GffGender), String>),
    CExoString(String),
    Char(i8),
    CResRef(String),
    Double(f64),
    DWord(u32),
    DWord64(u64),
    Float(f32),
    Int(i32),
    Int64(i64),
    Short(i16),
    Void(Vec<u8>),
    Word(u16),
    Struct(GffStruct),
    List(Vec<GffStruct>),
    Invalid,
}

#[derive(PartialEq)]
pub struct GffStruct {
    pub fields: HashMap<String, GffFieldValue>,
}

impl std::fmt::Debug for GffStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = self.fields.keys().collect::<Vec<&String>>();
        keys.sort();
        let mut res = f.debug_struct("GffStruct");

        for key in keys {
            res.field(key, &*self.fields.get(key).unwrap());
        }
        res.finish()
    }
}

/* }}} */
/* {{{ Encodings */

pub enum Encodings {
    NeverwinterNights,
}

pub type EncodingFn = dyn Fn(Option<u32>)
-> Result<&'static encoding_rs::Encoding, &'static str>;

impl std::ops::Deref for Encodings {
    type Target = EncodingFn;
    fn deref(&self) -> &Self::Target {
        match self {
            Encodings::NeverwinterNights => &|lang: Option<u32>| {
                match lang {
                    // used for CExoString
                    None => Ok(WINDOWS_1252),
                    // used for CExoLocString
                    Some(0) => Ok(WINDOWS_1252),
                    Some(1) => Ok(WINDOWS_1252),
                    Some(2) => Ok(WINDOWS_1252),
                    Some(3) => Ok(WINDOWS_1252),
                    Some(4) => Ok(WINDOWS_1252),
                    Some(5) => Ok(WINDOWS_1250),
                    Some(128) => Ok(EUC_KR),
                    Some(129) => Ok(BIG5),
                    Some(130) => Ok(GBK),
                    Some(131) => Ok(SHIFT_JIS),
                    _ => Err("Unknown lang"),
                }
            }
        }
    }
}

/* }}} */

/** UnpackStruct trait.
 *
 * This trait should be implemented for every structure
 * that can be unpack from GFF, by deriving gff_derive::DeGFF
 */
pub trait UnpackStruct {
    fn unpack(from: &GffStruct)
        -> Result<Self, &'static str> where Self: std::marker::Sized;
}
