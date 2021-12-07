use std::convert::TryFrom;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};


use nom::{
    IResult,
    number::complete::{
        le_u8, le_u16, le_u32, le_u64,
        le_i8, le_i16, le_i32, le_i64,
        le_f32, le_f64,
    },
    sequence::tuple,
    multi::count,
    bytes::complete::{ take, take_till },
    combinator::{
        map, map_res, map_parser, verify,
    },
    Err,
};

use crate::common::{
    GffStruct,
    GffFieldValue,
    GffHeader,
    GffGender,
    GffLang,
    OffsetCount,
};

pub struct GffParser {
    /* ensure we only parse a struct once, loops are not allowed */
    visited_structs: HashSet<u32>,
}

impl <'a> GffParser {
    pub fn parse(data: Vec<u8>, expected_type: &'a str)
        -> Result<GffStruct, Err<nom::error::Error<()>>>
    {
        let mut parser = GffParser {
            visited_structs: HashSet::new(),
        };
        let (_, header) = parser.parse_header(&data).map_err(|x| x.map_input(|_| ()))?;
        let (_, res) = parser.parse_struct(&header, &data, 0)
            .map_err(|x| x.map_input(|_| ()))?;
        Ok(res)
    }

    fn parse_header(&self, data: &'a [u8])
        -> IResult<&'a [u8], GffHeader>
    {
        let (input, gff_type) = take(4usize)(data)?;
        let (input, version) = take(4usize)(input)?;
        let (input, (st_offset, st_count)) = tuple((le_u32, le_u32))(input)?;
        let (input, (f_offset, f_count)) = tuple((le_u32, le_u32))(input)?;
        let (input, (lbl_offset, lbl_count)) = tuple((le_u32, le_u32))(input)?;
        let (input, (fd_offset, fd_count)) = tuple((le_u32, le_u32))(input)?;
        let (input, (fi_offset, fi_count)) = tuple((le_u32, le_u32))(input)?;
        let (input, (li_offset, li_count)) = tuple((le_u32, le_u32))(input)?;

        Ok((input, GffHeader {
            gff_type: [gff_type[0], gff_type[1], gff_type[2], gff_type[3]],
            version: [version[0], version[1],version[2],version[3]],
            structs: OffsetCount(st_offset, st_count),
            fields: OffsetCount(f_offset, f_count),
            labels: OffsetCount(lbl_offset, lbl_count),
            field_data: OffsetCount(fd_offset, fd_count),
            field_indices: OffsetCount(fi_offset, fi_count),
            list_indices: OffsetCount(li_offset, li_count),
        }))
    }

    fn parse_struct(&mut self, header: &GffHeader, data: &'a [u8], st_idx: u32)
        -> IResult<&'a [u8], GffStruct>
    {
        assert!(st_idx < header.structs.1);

        let start = header.structs.0 + (12 * st_idx);
        let (input, _) = take(start as usize)(data)?;
        let (input, (_st_type, field_offset, field_count)) = tuple((le_u32, le_u32, le_u32))(input)?;

        match field_count {
            0 => Ok((input, GffStruct { fields: HashMap::new() })),
            1 => {
                let (_, field) = self.parse_field(header, data, field_offset)?;
                Ok((input, GffStruct { fields: vec![field].into_iter().collect() }))
            },
            _ => {
                let (input, fields) = self.parse_field_indices(
                    header, data, field_offset, field_count as usize)?;
                Ok((input, GffStruct { fields: fields }))
            },
        }
    }

    fn parse_field(&mut self, header: &GffHeader, data: &'a [u8], f_idx: u32)
        -> IResult<&'a [u8], (String, GffFieldValue)>
    {
        assert!(f_idx < header.fields.1);
        let start = header.fields.0 + (12 * f_idx);
        let (input, _) = take(start as usize)(data)?;
        let (input, (gff_type, lbl_idx)) = tuple((le_u32, le_u32))(input)?;
        let value = match gff_type {
            0 => {
                let (_, val) = le_u8(input)?;
                GffFieldValue::Byte(val)
            },
            1 => {
                let (_, val) = le_i8(input)?;
                GffFieldValue::Char(val)
            },
            2 => {
                let (_, val) = le_u16(input)?;
                GffFieldValue::Word(val)
            },
            3 => {
                let (_, val) = le_i16(input)?;
                GffFieldValue::Short(val)
            },
            4 => {
                let (_, val) = le_u32(input)?;
                GffFieldValue::DWord(val)
            },
            5 => {
                let (_, val) = le_i32(input)?;
                GffFieldValue::Int(val)
            },
            6 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_dword64(&header, data, offset)?;
                val
            },
            7 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_int64(&header, data, offset)?;
                val
            }
            8 => {
                let (_, val) = le_f32(input)?;
                GffFieldValue::Float(val)
            },
            9 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_double(&header, data, offset)?;
                val
            }
            10 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_cexostring(&header, data, offset)?;
                val
            }
            11 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_cresref(&header, data, offset)?;
                val
            }
            12 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_cexosloctring(&header, data, offset)?;
                val
            }
            13 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_void(&header, data, offset)?;
                val
            }
            14 => {
                let visited_structs = Arc::new(Mutex::new(&mut self.visited_structs));

                let (_, st_idx) = verify(
                    le_u32,
                    |val: &u32| visited_structs.lock().unwrap().insert(*val))(input)?;
                let (_, val) = self.parse_struct(&header, data, st_idx)?;
                GffFieldValue::Struct(val)
            }
            15 => {
                let (_, li_idx) = le_u32(input)?;
                let (_, val) = self.parse_list(&header, data, li_idx)?;
                GffFieldValue::List(val)
            }
            bad => {
                println!("bad field type: {}", bad);
                GffFieldValue::Invalid
            }
        };
        let (input, label) = self.parse_label(header, data, lbl_idx)?;
        Ok((input, (label, value)))
    }

    fn parse_dword64(&self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, val) = le_u64(input)?;
        Ok((input, GffFieldValue::DWord64(val)))
    }

    fn parse_double(&self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, val) = le_f64(input)?;
        Ok((input, GffFieldValue::Double(val)))
    }

    fn parse_int64(&self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, val) = le_i64(input)?;
        Ok((input, GffFieldValue::Int64(val)))
    }

    fn parse_cexostring(&self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, len) = le_u32(input)?;
        let (input, s) = map(
            take(len as usize),
            |slice: &[u8]| slice.iter().map(|&c| c as char).collect()

        )(input)?;
        Ok((input, GffFieldValue::CExoString(s)))
    }

    fn parse_cresref(&self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, len) = le_u8(input)?;
        let (input, s) = map(
            take(len as usize),
            |slice: &[u8]| slice.iter().map(|&c| c as char).collect()
        )(input)?;
        Ok((input, GffFieldValue::CResRef(s)))
    }

    fn parse_cexosloctring(&self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, len) = le_u32(input)?;
        let (input, tlk_ref) = le_u32(input)?;
        let (input, str_count) = le_u32(input)?;

        fn parse_substring(data: &[u8])
            -> IResult<&[u8], (GffLang, GffGender, String)>
        {
            let (input, (lang, gender)) = map_res(le_u32, |val: u32|
                -> Result<(GffLang, GffGender), num_enum::TryFromPrimitiveError<_>> {
                    let gender = if val % 2 == 0 { GffGender::Male } else { GffGender::Female };
                    let lang = GffLang::try_from(val / 2)?;
                    Ok((lang, gender))
                })(data)?;
            let (input, len) = le_u32(input)?;
            let (input, s) = map(
                take(len as usize),
                |slice: &[u8]| slice.iter().map(|&c| c as char).collect()
            )(input)?;
            Ok((input, (lang, gender, s)))
        }

        let (input, s) = map_parser(
            take(len as usize - 8),
            count(parse_substring, str_count as usize)
        )(input)?;

        let mut locs = HashMap::new();
        for (lang, gender, subs) in s {
            locs.insert((lang, gender), subs);
        }
        Ok((input, GffFieldValue::CExoLocString(tlk_ref, locs)))
    }

    fn parse_void(&self, header: &GffHeader, data: &'a [u8], offset:u32)
        -> IResult<&'a [u8], GffFieldValue>
    {
        let start = header.field_data.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, len) = le_u32(input)?;
        let (input, data) = take(len as usize)(input)?;
        Ok((input, GffFieldValue::Void(data.to_vec())))
    }

    fn parse_list(&mut self, header: &GffHeader, data: &'a [u8], offset: u32)
        -> IResult<&'a [u8], Vec<GffStruct>>
    {
        assert!(offset % 4 == 0);

        let start = header.list_indices.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, list_size) = le_u32(input)?;

        let wself = Arc::new(Mutex::new(self));
        let (input, structs) =
            count(
                map_res(
                    verify(le_u32, |val: &u32| {
                        wself.lock().unwrap().visited_structs.insert(*val)
                    }),
                    |st_idx: u32| -> Result<GffStruct, Err<nom::error::Error<&[u8]>>> {
                        let (_, val) = wself.lock().unwrap().parse_struct(&header, data, st_idx)?;
                        Ok(val)
                    }),
                    list_size as usize
            )(input)?;
        Ok((input, structs))
    }

    fn parse_field_indices(&mut self, header: &GffHeader, data: &'a [u8], offset: u32, f_count: usize)
        -> IResult<&'a [u8], HashMap<String, GffFieldValue>>
    {
        assert!(offset % 4 == 0);
        assert!(offset < header.field_indices.1);
        let start = header.field_indices.0 + offset;
        let (input, _) = take(start as usize)(data)?;
        let (input, fields) =
            count(map_res(le_u32, |f_idx: u32|
                    -> Result<(String, GffFieldValue), Err<nom::error::Error<&[u8]>>>
            {
                let (_, val) = self.parse_field(&header, data, f_idx)?;
                Ok(val)
            }),
            f_count
            )(input)?;

        Ok((input, fields.into_iter().collect()))
    }

    fn parse_label(&self, header: &GffHeader, data: &'a [u8], lbl_idx: u32)
        -> IResult<&'a [u8], String>
    {
        let start = header.labels.0 + 16 * lbl_idx;
        let (input, _) = take(start as usize)(data)?;
        let (input, s) = map_res(
            map_parser(take(16usize), take_till(|c| c == 0x00)),
            |slice: &[u8]| String::from_utf8(slice.to_vec())
        )(input)?;
        Ok((input, s))
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::fs::File;
    use crate::parser::GffParser;
    use crate::common::GffStruct;

    fn test_parse(filename: &str)
        -> std::result::Result<GffStruct, Box<dyn std::error::Error>>
    {
        let mut f = File::open(filename)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        let res = GffParser::parse(buffer, "")?;
        Ok(res)
    }
    #[test]
    fn test_01_parse_gff_sample() {
        let res = test_parse("test-data/test.bic");
        assert!(res.is_ok())
    }
}
