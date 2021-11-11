extern crate num_enum;

use std::collections::HashMap;

/* {{{ GFF header */

#[derive(Debug)]
/** Tuple containing an offset, and a count */
pub struct OffsetCount (pub u32, pub u32);

#[derive(Debug)]
pub struct GffHeader<'a> {
    pub gff_type: &'a str,
    pub version: &'a str,
    pub structs: OffsetCount,
    pub fields: OffsetCount,
    pub labels: OffsetCount,
    pub field_data: OffsetCount,
    pub field_indices: OffsetCount,
    pub list_indices: OffsetCount,
}

/* }}} */

#[derive(Debug, std::cmp::Eq, std::cmp::PartialEq,
    std::hash::Hash, num_enum::TryFromPrimitive)]
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

#[derive(Debug, std::cmp::Eq, std::cmp::PartialEq,
    std::hash::Hash, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum GffGender {
    Male = 0,
    Female = 1,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GffStruct {
    pub fields: HashMap<String, GffFieldValue>,
}

/** UnpackStruct trait.
 *
 * This trait should be implemented for every structure
 * that can be unpack from GFF, by deriving gff_derive::DeGFF
 */
pub trait UnpackStruct {
    fn unpack(from: &GffStruct)
        -> Result<Self, &'static str> where Self: std::marker::Sized;
}
