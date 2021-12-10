#[cfg(test)]
mod tests {
    use gff::common::GffStruct;
    use gff::common::GffFieldValue;
    use std::collections::HashMap;
    use gff::common::UnpackStruct;
    use gff_derive;

    type Error = &'static str;

    #[test]
    fn test_float_double() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct1 {
            a1: f32,
            b1: f64,
            b2: f64,
        }

        let mut s = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::new(),
        };
        s.fields.insert(String::from("a1"), GffFieldValue::Float(1.5));
        s.fields.insert(String::from("b1"), GffFieldValue::Float(2.0));
        s.fields.insert(String::from("b2"), GffFieldValue::Double(3.14));

        assert_eq!(TestStruct1::unpack(&s).unwrap(), TestStruct1 { a1: 1.5, b1: 2.0, b2: 3.14 });
    }

    #[test]
    fn test_uint() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct2 {
            a1: u8,

            b1: u16,
            b2: u16,

            c1: u32,
            c2: u32,
            c3: u32,

            d1: u64,
            d2: u64,
            d3: u64,
            d4: u64,
        }

        let mut s = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::new(),
        };
        s.fields.insert(String::from("a1"), GffFieldValue::Byte(1));
        s.fields.insert(String::from("b1"), GffFieldValue::Byte(1));
        s.fields.insert(String::from("c1"), GffFieldValue::Byte(1));
        s.fields.insert(String::from("d1"), GffFieldValue::Byte(1));

        s.fields.insert(String::from("b2"), GffFieldValue::Word(2));
        s.fields.insert(String::from("c2"), GffFieldValue::Word(2));
        s.fields.insert(String::from("d2"), GffFieldValue::Word(2));

        s.fields.insert(String::from("c3"), GffFieldValue::DWord(3));
        s.fields.insert(String::from("d3"), GffFieldValue::DWord(3));

        s.fields.insert(String::from("d4"), GffFieldValue::DWord64(4));
        assert_eq!(TestStruct2::unpack(&s).unwrap(), TestStruct2 {
            a1: 1, b1: 1, c1: 1, d1: 1,
            b2: 2, c2: 2, d2: 2,
            c3: 3, d3: 3,
            d4: 4
        });
    }

    #[test]
    fn test_int() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct2 {
            a1: i8,

            b1: i16,
            b2: i16,

            c1: i32,
            c2: i32,
            c3: i32,

            d1: i64,
            d2: i64,
            d3: i64,
            d4: i64,
        }

        let mut s = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::new()
        };
        s.fields.insert(String::from("a1"), GffFieldValue::Char(-1));
        s.fields.insert(String::from("b1"), GffFieldValue::Char(-1));
        s.fields.insert(String::from("c1"), GffFieldValue::Char(-1));
        s.fields.insert(String::from("d1"), GffFieldValue::Char(-1));

        s.fields.insert(String::from("b2"), GffFieldValue::Short(-2));
        s.fields.insert(String::from("c2"), GffFieldValue::Short(-2));
        s.fields.insert(String::from("d2"), GffFieldValue::Short(-2));

        s.fields.insert(String::from("c3"), GffFieldValue::Int(-3));
        s.fields.insert(String::from("d3"), GffFieldValue::Int(-3));

        s.fields.insert(String::from("d4"), GffFieldValue::Int64(-4));
        assert_eq!(TestStruct2::unpack(&s).unwrap(), TestStruct2 {
            a1: -1, b1: -1, c1: -1, d1: -1,
            b2: -2, c2: -2, d2: -2,
            c3: -3, d3: -3,
            d4: -4
        });
    }

    #[test]
    fn test_sub_struct() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestSubStruct3 {
            a: i8,
            b: u8,
            c: i16,
            d: u16,
            e: i32,
            f: u32,
            g: i64,
            h: u64,
        }
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct3 {
            a: TestSubStruct3,
        }
        let mut s = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::new(),
        };
        let mut ss = GffStruct {
            st_type: 0x55555555,
            fields: HashMap::new()
        };
        ss.fields.insert(String::from("a"), GffFieldValue::Char(-1));
        ss.fields.insert(String::from("b"), GffFieldValue::Byte(1));
        ss.fields.insert(String::from("c"), GffFieldValue::Short(-1));
        ss.fields.insert(String::from("d"), GffFieldValue::Word(1));
        ss.fields.insert(String::from("e"), GffFieldValue::Int(-1));
        ss.fields.insert(String::from("f"), GffFieldValue::DWord(1));
        ss.fields.insert(String::from("g"), GffFieldValue::Int64(-1));
        ss.fields.insert(String::from("h"), GffFieldValue::DWord64(1));

        s.fields.insert(String::from("a"), GffFieldValue::Struct(ss));

        assert_eq!(TestStruct3::unpack(&s).unwrap(), TestStruct3 {
            a: TestSubStruct3 {
                a: -1,
                b: 1,
                c: -1,
                d: 1,
                e: -1,
                f: 1,
                g: -1,
                h: 1,
            }
        })
    }

    #[test]
    fn test_sub_struct_vec() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestSubStruct4 {
            a: i8,
        }
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct4 {
            a: Vec<TestSubStruct4>,
        }
        let mut s = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::new(),
        };
        let mut ss = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::new(),
        };
        ss.fields.insert(String::from("a"), GffFieldValue::Char(-1));
        s.fields.insert(String::from("a"), GffFieldValue::List(vec![ss]));

        assert_eq!(TestStruct4::unpack(&s).unwrap(), TestStruct4 {
            a: vec![TestSubStruct4 { a: -1 }]
        })
    }
}
