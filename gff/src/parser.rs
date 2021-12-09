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
        all_consuming,
    },
};

use crate::common::{
    GffStruct,
    GffFieldValue,
    GffHeader,
    GffGender,
    GffLang,
    OffsetCount,
    EncodingFn,
};

/** Header and data blocks of GFF file. */
struct Data<'a> {
    header: GffHeader,
    structs: &'a [u8],
    fields: &'a [u8],
    labels: &'a [u8],
    field_data: &'a [u8],
    field_indices: &'a [u8],
    list_indices: &'a [u8],
}

pub struct GffParser<'a> {
    /* ensure we only parse a struct once, loops are not allowed */
    visited_structs: HashSet<u32>,
    encodings: &'a EncodingFn,
}

type GResult<'a, T> = IResult<&'a [u8], T>;

impl <'a, 'b> GffParser<'b> {
    pub fn parse(data: Vec<u8>, encodings: &'b EncodingFn)
        -> Result<GffStruct, String>
    {
        let mut parser = GffParser {
            visited_structs: HashSet::new(),
            encodings: encodings,
        };
        let (_, data) = parser.parse_header(&data)
            .map_err(|e| format!("error parsing header: {:#?}", e))?;
        let (_, res) = parser.parse_struct(&data, 0)
            .map_err(|e| format!("error parsing data: {:#?}", e))?;
        Ok(res)
    }

    fn parse_header(&self, data: &'a [u8])
        -> GResult<'a, Data<'a>>
    {
        let header_size: u32 = 14 * 4;
        let mut data_offset = header_size;
        let st_size = 12;
        let f_size = 12;
        let lbl_size = 16;

        let (input, header_data) = take(header_size as usize)(data)?;
        let (header_data, gff_type) = take(4usize)(header_data)?;
        let (header_data, version) = take(4usize)(header_data)?;
        let (header_data, (st_offset, st_count)) = tuple((
            verify(le_u32, |val: &u32| { *val == data_offset }),
            le_u32
        ))(header_data)?;
        let (input, structs) = take(st_count * st_size)(input)?;
        data_offset += structs.len() as u32;

        let (header_data, (f_offset, f_count)) = tuple((
                verify(le_u32, |val: &u32| { *val == data_offset }),
                le_u32
        ))(header_data)?;
        let (input, fields) = take(f_count * f_size)(input)?;
        data_offset += fields.len() as u32;

        let (header_data, (lbl_offset, lbl_count)) = tuple((
                verify(le_u32, |val: &u32| { *val == data_offset }),
                le_u32
        ))(header_data)?;
        let (input, labels) = take(lbl_count * lbl_size)(input)?;
        data_offset += labels.len() as u32;

        let (header_data, (fd_offset, fd_count)) = tuple((
                verify(le_u32, |val: &u32| { *val == data_offset }),
                le_u32
        ))(header_data)?;
        let (input, field_data) = take(fd_count)(input)?;
        data_offset += field_data.len() as u32;

        let (header_data, (fi_offset, fi_count)) = tuple((
                verify(le_u32, |val: &u32| { *val == data_offset }),
                le_u32
        ))(header_data)?;
        let (input, field_indices) = take(fi_count)(input)?;
        data_offset += field_indices.len() as u32;

        let (_, (li_offset, li_count)) = all_consuming(tuple((
                    verify(le_u32, |val: &u32| { *val == data_offset}),
                    le_u32
        )))(header_data)?;
        let (input, list_indices) = all_consuming(take(li_count))(input)?;

        let parsed_header = GffHeader {
            gff_type: [gff_type[0], gff_type[1], gff_type[2], gff_type[3]],
            version: [version[0], version[1],version[2],version[3]],
            structs: OffsetCount(st_offset, st_count),
            fields: OffsetCount(f_offset, f_count),
            labels: OffsetCount(lbl_offset, lbl_count),
            field_data: OffsetCount(fd_offset, fd_count),
            field_indices: OffsetCount(fi_offset, fi_count),
            list_indices: OffsetCount(li_offset, li_count),
        };


        Ok((input, Data {
            header: parsed_header,
            structs: structs,
            fields: fields,
            labels: labels,
            field_data: field_data,
            field_indices: field_indices,
            list_indices: list_indices,
        }))
    }

    fn parse_struct(&mut self, data: &'a Data, st_idx: u32)
        -> GResult<'a, GffStruct>
    {
        assert!(st_idx < data.header.structs.1);

        let (input, _) = take(12 * st_idx as usize)(data.structs)?;
        let (input, (_st_type, field_offset, field_count)) = tuple((le_u32, le_u32, le_u32))(input)?;

        match field_count {
            0 => Ok((input, GffStruct { fields: HashMap::new() })),
            1 => {
                let (_, field) = self.parse_field(data, field_offset)?;
                Ok((b"", GffStruct { fields: vec![field].into_iter().collect() }))
            },
            _ => {
                let (input, fields) = self.parse_field_indices(
                    data, field_offset, field_count as usize)?;
                Ok((input, GffStruct { fields: fields }))
            },
        }
    }

    fn parse_field(&mut self, data: &'a Data, f_idx: u32)
        -> GResult<'a, (String, GffFieldValue)>
    {
        assert!(f_idx < data.header.fields.1);
        let (input, _) = take(12 * f_idx)(data.fields)?;
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
                let (_, val) = self.parse_dword64(data, offset)?;
                val
            },
            7 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_int64(data, offset)?;
                val
            }
            8 => {
                let (_, val) = le_f32(input)?;
                GffFieldValue::Float(val)
            },
            9 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_double(data, offset)?;
                val
            }
            10 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_cexostring(data, offset)?;
                val
            }
            11 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_cresref(data, offset)?;
                val
            }
            12 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_cexosloctring(data, offset)?;
                val
            }
            13 => {
                let (_, offset) = le_u32(input)?;
                let (_, val) = self.parse_void(data, offset)?;
                val
            }
            14 => {
                let visited_structs = Arc::new(Mutex::new(&mut self.visited_structs));

                let (_, st_idx) = verify(
                    le_u32,
                    |val: &u32| visited_structs.lock().unwrap().insert(*val))(input)?;
                let (_, val) = self.parse_struct(data, st_idx)?;
                GffFieldValue::Struct(val)
            }
            15 => {
                let (_, li_idx) = le_u32(input)?;
                let (_, val) = self.parse_list(data, li_idx)?;
                GffFieldValue::List(val)
            }
            bad => {
                println!("bad field type: {}", bad);
                GffFieldValue::Invalid
            }
        };
        let (input, label) = self.parse_label(data, lbl_idx)?;
        Ok((input, (label, value)))
    }

    fn parse_dword64(&self, data: &'a Data, offset: u32)
        -> GResult<'a, GffFieldValue>
    {
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, val) = le_u64(input)?;
        Ok((input, GffFieldValue::DWord64(val)))
    }

    fn parse_double(&self, data: &'a Data, offset: u32)
        -> GResult<'a, GffFieldValue>
    {
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, val) = le_f64(input)?;
        Ok((input, GffFieldValue::Double(val)))
    }

    fn parse_int64(&self, data: &'a Data, offset: u32)
        -> GResult<'a, GffFieldValue>
    {
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, val) = le_i64(input)?;
        Ok((input, GffFieldValue::Int64(val)))
    }

    fn parse_cexostring(&self, data: &'a Data, offset: u32)
        -> GResult<'a, GffFieldValue>
    {
        let encodings = self.encodings;
        let encoding = encodings(None).unwrap();
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, len) = le_u32(input)?;
        let (input, (s, _, _)) = map(
            take(len as usize),
            |slice: &[u8]| encoding.decode(slice)
        )(input)?;
        Ok((input, GffFieldValue::CExoString(s.to_string())))
    }

    fn parse_cresref(&self, data: &'a Data, offset: u32)
        -> GResult<'a, GffFieldValue>
    {
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, len) = le_u8(input)?;
        let (input, s) = map(
            take(len as usize),
            |slice: &[u8]| slice.iter().map(|&c| c as char).collect()
        )(input)?;
        Ok((input, GffFieldValue::CResRef(s)))
    }

    fn parse_cexosloctring(&self, data: &'a Data, offset: u32)
        -> GResult<'a, GffFieldValue>
    {
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, len) = le_u32(input)?;
        let (input, tlk_ref) = le_u32(input)?;
        let (input, str_count) = le_u32(input)?;

        let encodings = self.encodings;

        fn parse_substring<'a>(data: &'a [u8], encodings: &EncodingFn)
            -> GResult<'a, (GffLang, GffGender, String)>
        {
            let (input, (lang, gender)) = map_res(le_u32, |val: u32|
                -> Result<(GffLang, GffGender), num_enum::TryFromPrimitiveError<_>> {
                    let gender = if val % 2 == 0 { GffGender::Male } else { GffGender::Female };
                    let lang = GffLang::try_from(val / 2)?;
                    Ok((lang, gender))
                })(data)?;
            let encoding = encodings(Some(lang as u32)).unwrap();
            let (input, len) = le_u32(input)?;

            let (input, slice) = take(len as usize)(input)?;
            let (s, _, _) = encoding.decode(slice);

            Ok((input, (lang, gender, s.to_string())))
        }

        let (input, s) = map_parser(
            take(len as usize - 8),
            count(|data| { parse_substring(data, encodings) }, str_count as usize)
        )(input)?;

        let mut locs = HashMap::new();
        for (lang, gender, subs) in s {
            locs.insert((lang, gender), subs);
        }
        Ok((input, GffFieldValue::CExoLocString(tlk_ref, locs)))
    }

    fn parse_void(&self, data: &'a Data, offset:u32)
        -> GResult<'a, GffFieldValue>
    {
        let (input, _) = take(offset as usize)(data.field_data)?;
        let (input, len) = le_u32(input)?;
        let (input, data) = take(len as usize)(input)?;
        Ok((input, GffFieldValue::Void(data.to_vec())))
    }

    fn parse_list(&mut self, data: &'a Data, offset: u32)
        -> GResult<'a, Vec<GffStruct>>
    {
        assert!(offset % 4 == 0);

        let (input, _) = take(offset as usize)(data.list_indices)?;
        let (input, list_size) = le_u32(input)?;

        let wself = Arc::new(Mutex::new(self));
        let (input, structs) =
            count(
                map_res(
                    verify(le_u32, |val: &u32| {
                        wself.lock().unwrap().visited_structs.insert(*val)
                    }),
                    |st_idx: u32| -> GResult<'a, GffStruct> {
                        let (input, val) = wself.lock().unwrap().parse_struct(data, st_idx)?;
                        Ok((input, val))
                    }),
                    list_size as usize
            )(input)?;
        let structs = structs.into_iter().map(|t| t.1).collect();
        Ok((input, structs))
    }

    fn parse_field_indices(&mut self, data: &'a Data, offset: u32, f_count: usize)
        -> GResult<'a, HashMap<String, GffFieldValue>>
    {
        assert!(offset % 4 == 0);
        assert!(offset < data.header.field_indices.1);
        let (input, _) = take(offset as usize)(data.field_indices)?;
        let (input, fields) = count(
            map_res(le_u32, |f_idx: u32|
                    -> GResult<'a, (String, GffFieldValue)> {
                let (input, val) = self.parse_field(data, f_idx)?;
                Ok((input, val))
            }),
            f_count
        )(input)?;

        Ok((input, fields.into_iter().map(|t| t.1).collect()))
    }

    fn parse_label(&self, data: &'a Data, lbl_idx: u32)
        -> GResult<'a, String>
    {
        let (input, _) = take(lbl_idx as usize * 16)(data.labels)?;
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
    use crate::common::{
        GffStruct,
        Encodings,
    };

    fn test_parse(filename: &str)
        -> std::result::Result<GffStruct, Box<dyn std::error::Error>>
    {
        let mut f = File::open(filename)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        let res = GffParser::parse(buffer, &*Encodings::NeverwinterNights)?;
        Ok(res)
    }
    #[test]
    fn test_01_parse_gff_sample() {
        let res = test_parse("test-data/test.bic");
        assert!(res.is_ok())
    }
}
