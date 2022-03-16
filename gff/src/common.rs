//! Common types used by GFF

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

/// Tuple containing an offset, and a count
///
/// This is used in the [`GffHeader`] to delimitate
/// different zones of packed data.
#[derive(Debug, Default)]
pub struct OffsetCount (pub u32, pub u32);

/// Unpacked version of the GFF header.
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

impl Default for GffHeader {
    fn default() -> Self {
        GffHeader {
            gff_type: *b"    ",
            version: *b"V3.2",
            structs: OffsetCount::default(),
            fields: OffsetCount::default(),
            labels: OffsetCount::default(),
            field_data: OffsetCount::default(),
            field_indices: OffsetCount::default(),
            list_indices: OffsetCount::default(),
        }
    }
}
/* }}} */
/* {{{ Gff Structs and fields */

/// Representation of a player language id
///
/// This is used to localize dialog strings
/// based on the player's selected language.
///
/// (see [`GffFieldValue::CExoLocString`])
///
/// Note that these IDs are only valid for
/// Neverwinter Nights, other games may use
/// other IDs.
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

/// Representation of a character gender
///
/// This is used to localize dialog strings
/// based on the player's character choice.
///
/// (see [`GffFieldValue::CExoLocString`])
#[derive(Debug, std::cmp::Eq, PartialEq,
    std::hash::Hash, num_enum::TryFromPrimitive,
    Copy, Clone)]
#[repr(u8)]
pub enum GffGender {
    Male = 0,
    Female = 1,
}

/// Intermediary representation of a packed struct field
#[derive(Debug, PartialEq)]
pub enum GffFieldValue {
    /// A basic [`u8`] value
    Byte(u8),
    /// A localized string
    CExoLocString(u32, HashMap<(GffLang, GffGender), String>),
    /// A non-localized string
    CExoString(String),
    /// A basic [`i8`] value
    Char(i8),
    /// A Resource Reference string
    CResRef(String),
    /// A basic [`f64`] value
    Double(f64),
    /// A basic [`u32`] value
    DWord(u32),
    /// A basic [`u64`] value
    DWord64(u64),
    /// A basic [`f32`] value
    Float(f32),
    /// A basic [`i32`] value
    Int(i32),
    /// A basic [`i64`] value
    Int64(i64),
    /// A basic [`i16`] value
    Short(i16),
    /// Raw data
    Void(Vec<u8>),
    /// A basic [`u16`] value
    Word(u16),
    /// Another struct
    Struct(GffStruct),
    /// A list of structs
    ///
    /// Note: vectors of other types cannot be packed
    /// and must be wrapped into a struct.
    List(Vec<GffStruct>),
    Invalid,
}

/// Intermediary representation of a packed struct
#[derive(PartialEq)]
pub struct GffStruct {
    pub st_type: u32,
    pub fields: HashMap<String, GffFieldValue>,
}

impl std::fmt::Debug for GffStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = self.fields.keys().collect::<Vec<&String>>();
        keys.sort();
        let mut res = f.debug_struct(&format!("GffStruct (0x{:x})", self.st_type));

        for key in keys {
            res.field(key, &*self.fields.get(key).unwrap());
        }
        res.finish()
    }
}

/* }}} */
/* {{{ Encodings */

/// Enum representing various game languages and their encodings
pub enum Encodings {
    NeverwinterNights,
}

/// Callback to match a language to the appropriate encoder/decoder
///
/// For a [`GffFieldValue::CExoLocString`], provide `Some(language_id)`
/// For a [`GffFieldValue::CExoString`], provide `None`
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
/* {{{ Public traits */

/// Deserialize trait.
///
/// Implement for any structure that should be deserializable from
/// [`GffStruct`] intermediary representation.
///
/// This trait can be automatically derived using
/// `gff_derive::GffStruct`, as long as all fields of
/// the struct implement [`crate::common::Deserialize`].
pub trait Deserialize {
    fn deserialize(from: &GffStruct)
        -> Result<Self, &'static str> where Self: std::marker::Sized;
}

/// Serialize trait.
///
/// Implement for any structure that should be serializable to
/// [`GffStruct`] intermediary representation.
///
/// This trait can be automatically derived using
/// `gff_derive::GffStruct`, as long as all fields of
/// the struct implement [`crate::common::Serialize`]
pub trait Serialize {
    fn serialize(&self)
        -> Result<GffStruct, &'static str> where Self: std::marker::Sized;
}

/* }}} */
