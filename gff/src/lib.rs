extern crate encoding_rs;

pub mod common;
pub mod deserialize;
pub mod serialize;
pub mod parser;
pub mod packer;


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::prelude::*;
    use std::fs::File;
    use crate::parser::GffParser;
    use crate::packer::Packer;
    use crate::common::{
        GffFieldValue,
        GffStruct,
        GffGender,
        GffLang,
        Encodings,
    };

    fn test_pack_unpack(input: &GffStruct) {
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);

        packer.pack(&input).unwrap();

        let data = packer.writer.into_inner().unwrap();
        let encoding = &*Encodings::NeverwinterNights;
        let res = GffParser::parse(data, encoding).unwrap();

        assert_eq!(res, *input);
    }

    fn test_1_field(val: GffFieldValue) {
        let val = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([(String::from("field1"), val)]),
        };
        test_pack_unpack(&val);
    }

    #[test]
    fn test_001_all_single_fields() {
        test_1_field(GffFieldValue::Byte(1));
        test_1_field(GffFieldValue::Char(1));
        test_1_field(GffFieldValue::Short(1));
        test_1_field(GffFieldValue::Word(1));
        test_1_field(GffFieldValue::Int(1));
        test_1_field(GffFieldValue::DWord(1));
        test_1_field(GffFieldValue::Int64(1));
        test_1_field(GffFieldValue::DWord64(1));
        test_1_field(GffFieldValue::Float(3.14));
        test_1_field(GffFieldValue::Double(3.14));
        test_1_field(GffFieldValue::CResRef(String::from("reference.bic")));
        test_1_field(GffFieldValue::CExoString(
                String::from("This is a sentence, hope you like it")));
        test_1_field(GffFieldValue::Void(b"qweasdzxc".to_vec()));
        test_1_field(GffFieldValue::CExoLocString(
                0xFFFFFFFF,
                HashMap::from([
                    ((GffLang::English, GffGender::Male), String::from("Hello sir")),
                    ((GffLang::English, GffGender::Female), String::from("Hello milady")),
                    ((GffLang::French, GffGender::Male), String::from("Salut bogosse")),
                    ((GffLang::French, GffGender::Female), String::from("Wesh madmazelle")),
                ])
        ));
        test_1_field(
            GffFieldValue::Struct(
                GffStruct {
                    st_type: 0xFFFFFFFF,
                    fields: HashMap::from([
                                (String::from("field2"), GffFieldValue::Byte(1))
                    ])
                }
            )
        );
        test_1_field(
            GffFieldValue::List(vec![
                GffStruct {
                    st_type: 0xFFFFFFFF,
                    fields: HashMap::from([
                                (String::from("field2"), GffFieldValue::Byte(0xAA))
                    ])
                }, GffStruct {
                    st_type: 0xFFFFFFFF,
                    fields: HashMap::from([
                                (String::from("field2"), GffFieldValue::Byte(0x55))
                    ])
                }]
            )
        );
    }

    // XXX: disabled for now, as we need to handle encodings
    #[test]
    fn test_002_pack_and_parse_sample() {
        let mut f = File::open("test-data/test.bic").unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let v1 = GffParser::parse(buffer, &*Encodings::NeverwinterNights).unwrap();
        test_pack_unpack(&v1);
    }
}
