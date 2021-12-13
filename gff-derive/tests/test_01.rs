#[cfg(test)]
mod tests {
    use gff::common::GffStruct;
    use gff::common::GffFieldValue;
    use std::collections::HashMap;
    use gff::common::Deserialize;
    use gff::common::Serialize;
    use gff_derive;
    use std::convert::TryInto;

    type Error = &'static str;

    macro_rules! test_serialize_deserialize {
        ( $type:ty, $struct: expr, $gff_struct: expr ) => {
            {
                let struct_v2 = <$type>::deserialize(&$gff_struct).unwrap();
                assert_eq!($struct, struct_v2);

                let gff_struct_v2 = &$struct.serialize().unwrap();
                assert_eq!($gff_struct, gff_struct_v2);
            }
        }
    }

    #[test]
    fn test_float_double() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct1 {
            a1: f32,
            b1: f64,
        }

        let gff_struct = GffStruct {
            st_type: 0x12345678,
            fields: HashMap::from([
                (String::from("a1"), GffFieldValue::Float(1.5)),
                (String::from("b1"), GffFieldValue::Double(2.0)),
            ]),
        };
        let struc = TestStruct1 {
            a1: 1.5, b1: 2.0,
        };

        test_serialize_deserialize!(TestStruct1, struc, &gff_struct)
    }

    #[test]
    fn test_uint() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct2 {
            a1: u8,
            b1: u16,
            c1: u32,
            d1: u64,
        }

        let gff_struct = GffStruct {
            st_type: 0x12345678,
            fields: HashMap::from([
                (String::from("a1"), GffFieldValue::Byte(1)),
                (String::from("b1"), GffFieldValue::Word(2)),
                (String::from("c1"), GffFieldValue::DWord(3)),
                (String::from("d1"), GffFieldValue::DWord64(4)),
            ]),
        };
        let struc = TestStruct2 {
            a1: 1,
            b1: 2,
            c1: 3,
            d1: 4,
        };
        test_serialize_deserialize!(TestStruct2, struc, &gff_struct);
    }

    #[test]
    fn test_int() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestStruct2 {
            a1: i8,
            b1: i16,
            c1: i32,
            d1: i64,
        }

        let gff_struct = GffStruct {
            st_type: 0x12345678,
            fields: HashMap::from([
                (String::from("a1"), GffFieldValue::Char(-1)),
                (String::from("b1"), GffFieldValue::Short(-2)),
                (String::from("c1"), GffFieldValue::Int(-3)),
                (String::from("d1"), GffFieldValue::Int64(-4)),
            ]),
        };
        let struc = TestStruct2 {
            a1: -1, b1: -2, c1: -3, d1: -4,
        };

        test_serialize_deserialize!(TestStruct2, struc, &gff_struct);
    }

    #[test]
    fn test_sub_struct() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x55555555)]
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
        #[GFFStructId(0xAAAAAAAA)]
        struct TestStruct3 {
            a: TestSubStruct3,
        }
        let ss = GffStruct {
            st_type: 0x55555555,
            fields: HashMap::from([
                (String::from("a"), GffFieldValue::Char(-1)),
                (String::from("b"), GffFieldValue::Byte(1)),
                (String::from("c"), GffFieldValue::Short(-1)),
                (String::from("d"), GffFieldValue::Word(1)),
                (String::from("e"), GffFieldValue::Int(-1)),
                (String::from("f"), GffFieldValue::DWord(1)),
                (String::from("g"), GffFieldValue::Int64(-1)),
                (String::from("h"), GffFieldValue::DWord64(1)),
            ]),
        };
        let gff_struct = GffStruct {
            st_type: 0xAAAAAAAA,
            fields: HashMap::from([
                (String::from("a"), GffFieldValue::Struct(ss)),
            ]),
        };

        let struc = TestStruct3 {
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
        };
        test_serialize_deserialize!(TestStruct3, struc, &gff_struct);
    }

    #[test]
    fn test_sub_struct_vec() {
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x12345678)]
        struct TestSubStruct4 {
            a: i8,
        }
        #[derive(gff_derive::DeGFF, std::cmp::PartialEq, Debug)]
        #[GFFStructId(0x87654321)]
        struct TestStruct4 {
            a: Vec<TestSubStruct4>,
        }
        let ss = GffStruct {
            st_type: 0x12345678,
            fields: HashMap::from([
                (String::from("a"), GffFieldValue::Char(-1)),
            ]),
        };
        let gff_struct = GffStruct {
            st_type: 0x87654321,
            fields: HashMap::from([
                (String::from("a"), GffFieldValue::List(vec![ss])),
            ]),
        };

        let struc = TestStruct4 {
            a: vec![TestSubStruct4 { a: -1 }]
        };
        test_serialize_deserialize!(TestStruct4, struc, &gff_struct);
    }
}
