use std::collections::HashMap;
use std::io::Write;

use crate::common::{
    GffHeader,
    GffStruct,
    GffFieldValue,
};

struct PackData {
    header: GffHeader,
    structs: Vec<u8>,
    fields: Vec<u8>,
    labels: Vec<u8>,
    field_data: Vec<u8>,
    field_indices: Vec<u8>,
    list_indices: Vec<u8>,
}

pub struct Packer<W: std::io::Write> {
    pub writer: std::io::BufWriter<W>,
    labels: HashMap<String, u32>,
    data: PackData,
}

impl PackData {
    fn new() -> Self {
        PackData {
            header: GffHeader::new(),
            structs: vec![],
            fields: vec![],
            labels: vec![],
            field_data: vec![],
            field_indices: vec![],
            list_indices: vec![],
        }
    }
}

impl <'b, W: std::io::Write> Packer<W> {
    pub fn new(writer: W) -> Packer<W> {
        Packer {
            writer: std::io::BufWriter::new(writer),
            labels: HashMap::new(),
            data: PackData::new(),
        }
    }

    /* write offsets into header */
    fn finalize(&mut self) {
        let mut offset = 14 * 4;

        self.data.header.structs.0 = offset;
        assert_eq!(self.data.header.structs.1 * 12, self.data.structs.len() as u32);
        offset += self.data.structs.len() as u32;

        self.data.header.fields.0 = offset;
        assert_eq!(self.data.header.fields.1 * 12, self.data.fields.len() as u32);
        offset += self.data.fields.len() as u32;

        self.data.header.labels.0 = offset;
        assert_eq!(self.data.header.labels.1 * 16, self.data.labels.len() as u32);
        offset += self.data.labels.len() as u32;

        self.data.header.field_data.0 = offset;
        assert_eq!(self.data.header.field_data.1, self.data.field_data.len() as u32);
        offset += self.data.field_data.len() as u32;

        self.data.header.field_indices.0 = offset;
        assert_eq!(self.data.header.field_indices.1, self.data.field_indices.len() as u32);
        offset += self.data.field_indices.len() as u32;

        self.data.header.list_indices.0 = offset;
        assert_eq!(self.data.header.list_indices.1, self.data.list_indices.len() as u32);
    }

    fn write(&mut self) {
        /* write header */
        self.writer.write(&self.data.header.gff_type);
        self.writer.write(&self.data.header.version);
        self.writer.write(&self.data.header.structs.0.to_le_bytes());
        self.writer.write(&self.data.header.structs.1.to_le_bytes());
        self.writer.write(&self.data.header.fields.0.to_le_bytes());
        self.writer.write(&self.data.header.fields.1.to_le_bytes());
        self.writer.write(&self.data.header.labels.0.to_le_bytes());
        self.writer.write(&self.data.header.labels.1.to_le_bytes());
        self.writer.write(&self.data.header.field_data.0.to_le_bytes());
        self.writer.write(&self.data.header.field_data.1.to_le_bytes());
        self.writer.write(&self.data.header.field_indices.0.to_le_bytes());
        self.writer.write(&self.data.header.field_indices.1.to_le_bytes());
        self.writer.write(&self.data.header.list_indices.0.to_le_bytes());
        self.writer.write(&self.data.header.list_indices.1.to_le_bytes());

        self.writer.write(&self.data.structs);
        self.writer.write(&self.data.fields);
        self.writer.write(&self.data.labels);
        self.writer.write(&self.data.field_data);
        self.writer.write(&self.data.field_indices);
        self.writer.write(&self.data.list_indices);
    }

    /* {{{ Pack functions */
    pub fn pack(&mut self, input: &'b GffStruct)
        -> Result<(), &'static str>
    {
        let mut structs: Vec<&GffStruct> = vec![input];
        let mut current_st_idx = 0;

        loop {
            let st_count = structs.len();
            if st_count == 0 {
                break;
            }
            let struct_to_write = structs.remove(0);
            self.pack_struct(&struct_to_write, &mut structs, &mut current_st_idx)?;
        }

        self.finalize();
        self.write();

        Ok(())
    }

    fn pack_struct(&mut self, input: &'b GffStruct,
        structs: &mut Vec<&'b GffStruct>, current_st_idx: &mut u32)
        -> Result<(), &'static str>
    {
        /* write struct type */
        self.data.structs.extend_from_slice(&0u32.to_le_bytes());
        match input.fields.len() {
            0 => {
                self.data.structs.extend_from_slice(
                    &(self.data.header.fields.1 as u32).to_le_bytes()
                );
                self.data.structs.extend_from_slice(
                    &0u32.to_le_bytes()
                );
            },
            1 => {
                self.data.structs.extend_from_slice(
                    &(self.data.header.fields.1 as u32).to_le_bytes()
                );
                self.data.structs.extend_from_slice(
                    &1u32.to_le_bytes()
                );
            },
            field_count => {
                self.data.structs.extend_from_slice(
                    &(self.data.header.field_indices.1 as u32).to_le_bytes()
                );
                self.data.structs.extend_from_slice(
                    &(field_count as u32).to_le_bytes()
                );
            }
        };

        self.data.header.structs.1 += 1;
        let mut field_indices = vec![];
        for (field, value) in &input.fields {
            let field_id = self.pack_field(
                &field, &value, structs, current_st_idx
            )?;

            field_indices.push(field_id);
        }

        if field_indices.len() > 1 {
            /* write fields indices into field_indices array */
            for field_indice in field_indices {
                self.data.field_indices.extend_from_slice(
                    &(field_indice as u32).to_le_bytes()
                );
                self.data.header.field_indices.1 += 4;
            }
        }
        Ok(())
    }

    fn pack_label(&mut self, label: &String)
        -> Result<u32, &'static str>
    {
        let max_label_idx = self.labels.len();
        let label_idx = self.labels
            .entry(label.clone())
            .or_insert(max_label_idx as u32);

        /* new label needs to be written */ 
        if *label_idx == self.data.header.labels.1 {
            self.data.header.labels.1 += 1;

            let label_data = label.clone().into_bytes();
            if label_data.len() > 16 {
                return Err("label too long");
            }

            self.data.labels.reserve(16);
            for i in 0..16 {
                if label_data.len() > i {
                    self.data.labels.push(label_data[i]);
                } else {
                    self.data.labels.push(0);
                }
            }
        }
        Ok(*label_idx)
    }

    fn pack_field(&mut self, field_name: &String, field_value: &'b GffFieldValue,
        structs: &mut Vec<&'b GffStruct>, current_st_idx: &mut u32)
        -> Result<u32, &'static str>
    {
        let label_idx = self.pack_label(field_name)?;

        self.data.header.fields.1 += 1;

        match field_value {
            GffFieldValue::Byte(val) => {
                self.pack_val_1(0, label_idx, *val);
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Char(val) => {
                self.pack_val_1(1, label_idx, *val as u8);
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Word(val) => {
                self.pack_val_2(2, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Short(val) => {
                self.pack_val_2(3, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::DWord(val) => {
                self.pack_val_4(4, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Int(val) => {
                self.pack_val_4(5, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::DWord64(val) => {
                self.pack_val_8(6, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Int64(val) => {
                self.pack_val_8(7, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Float(val) => {
                self.pack_val_4(8, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Double(val) => {
                self.pack_val_8(9, label_idx, &val.to_le_bytes());
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::CExoString(s) => {
                self.pack_data_offset(10, label_idx);

                let str_data = s.as_bytes();
                self.pack_data_u32(str_data.len() as u32);
                self.pack_data_slice(&str_data);
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::CResRef(s) => {
                self.pack_data_offset(11, label_idx);

                let s = s.to_lowercase();
                let str_data = s.as_bytes();
                if str_data.len() > 16 {
                    Err("ResRef too long")
                } else {
                    self.data.field_data.push(str_data.len() as u8);
                    self.data.header.field_data.1 += 1;
                    self.pack_data_slice(&str_data);
                    Ok(self.data.header.fields.1 - 1)
                }
            }
            GffFieldValue::CExoLocString(str_ref, val) => {
                self.pack_data_offset(12, label_idx);

                // string ref + string count
                let mut total_len: u32 = 8;

                for ((_lang, _gender), s) in val {
                    let s_vec = s.as_bytes();
                    // gender-lang + length + string
                    total_len += 8 + s_vec.len() as u32;
                }

                // total data size
                self.pack_data_u32(total_len);
                // string ref
                self.pack_data_u32(*str_ref);
                // string count
                self.pack_data_u32(val.len() as u32);

                for ((lang, gender), s) in val {
                    let s_vec = s.as_bytes();
                    let gender = *gender as u32;
                    let lang = *lang as u32;
                    // gender-lang
                    self.pack_data_u32(gender + 2 * lang);
                    // length
                    self.pack_data_u32(s_vec.len() as u32);
                    // string
                    self.pack_data_slice(s_vec);

                }
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Void(val) => {
                self.pack_data_offset(13, label_idx);

                self.pack_data_u32(val.len() as u32);
                self.pack_data_slice(val);
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::Struct(st) => {
                *current_st_idx += 1;
                self.pack_val_4(14, label_idx,
                    &(*current_st_idx).to_le_bytes()
                );
                structs.push(&st);
                Ok(self.data.header.fields.1 - 1)
            }
            GffFieldValue::List(vec) => {
                self.pack_val_4(15, label_idx,
                    &self.data.header.list_indices.1.to_le_bytes());
                self.pack_list_u32(vec.len() as u32);
                for st in vec {
                    *current_st_idx += 1;
                    self.pack_list_u32(*current_st_idx);
                    structs.push(&st);
                }
                Ok(self.data.header.fields.1 - 1)
            }
            _ => Err("Not handled yet")
        }
    }

    fn pack_val_1(&mut self, ftype: u32, label_idx: u32, val: u8) {
        self.data.fields.extend_from_slice(&ftype.to_le_bytes());
        self.data.fields.extend_from_slice(&label_idx.to_le_bytes());
        self.data.fields.push(val);
        self.data.fields.extend_from_slice(&[0u8; 3]);
    }

    fn pack_val_2(&mut self, ftype: u32, label_idx: u32, val: &[u8; 2]) {
        self.data.fields.extend_from_slice(&ftype.to_le_bytes());
        self.data.fields.extend_from_slice(&label_idx.to_le_bytes());
        self.data.fields.extend_from_slice(val);
        self.data.fields.extend_from_slice(&[0u8; 2]);
    }

    fn pack_val_4(&mut self, ftype: u32, label_idx: u32, val: &[u8; 4]) {
        self.data.fields.extend_from_slice(&ftype.to_le_bytes());
        self.data.fields.extend_from_slice(&label_idx.to_le_bytes());
        self.data.fields.extend_from_slice(val);
    }

    fn pack_data_offset(&mut self, ftype: u32, label_idx: u32) {
        self.pack_val_4(ftype, label_idx,
            &(self.data.field_data.len() as u32).to_le_bytes()
        );
    }

    fn pack_val_8(&mut self, ftype: u32, label_idx: u32, val: &[u8; 8]) {
        self.pack_data_offset(ftype, label_idx);
        self.data.field_data.extend_from_slice(val);
        self.data.header.field_data.1 += 8;
    }

    fn pack_data_u32(&mut self, val: u32) {
        self.data.field_data.extend_from_slice(&val.to_le_bytes());
        self.data.header.field_data.1 += 4;
    }
    fn pack_data_slice(&mut self, val: &[u8]) {
        self.data.field_data.extend_from_slice(val);
        self.data.header.field_data.1 += val.len() as u32;
    }

    fn pack_list_u32(&mut self, val: u32) {
        self.data.list_indices.extend_from_slice(&val.to_le_bytes());
        self.data.header.list_indices.1 += 4;
    }

    /* }}} */
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::packer::Packer;
    use crate::common::{
        GffStruct,
        GffFieldValue,
        GffLang,
        GffGender,
    };

    fn assert_struct_count(packer: &Packer<Vec<u8>>, st_count: usize) {
        assert_eq!(packer.data.header.structs.1, st_count as u32);
        // 3 DWORDS per struct
        assert_eq!(packer.data.structs.len(), st_count * 12);
    }
    fn assert_field_count(packer: &Packer<Vec<u8>>, f_count: usize) {
        assert_eq!(packer.data.header.fields.1, f_count as u32);
        // 3 DWORDS per field
        assert_eq!(packer.data.fields.len(), f_count * 12);
    }
    fn assert_field_indice_count(packer: &Packer<Vec<u8>>, fi_count: usize) {
        assert_eq!(packer.data.header.field_indices.1, fi_count as u32 * 4);
        // 1 DWORDS per field
        assert_eq!(packer.data.field_indices.len(), fi_count * 4);
    }
    fn assert_label_count(packer: &Packer<Vec<u8>>, l_count: usize) {
        assert_eq!(packer.data.header.labels.1, l_count as u32);
        // 16 bytes per label
        assert_eq!(packer.data.labels.len(), l_count * 16);
    }
    fn assert_field_data_count(packer: &Packer<Vec<u8>>, fd_count: usize) {
        assert_eq!(packer.data.header.field_data.1, fd_count as u32);
        assert_eq!(packer.data.field_data.len(), fd_count);
    }
    fn assert_list_count(packer: &Packer<Vec<u8>>, bytes: usize) {
        assert_eq!(packer.data.header.list_indices.1, bytes as u32);
        assert_eq!(packer.data.list_indices.len(), bytes);
    }

    #[test]
    fn test_01_pack_1_simple_field() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Byte(1)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 1);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 1);
    }

    #[test]
    fn test_02_pack_2_simple_fields() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Byte(1)),
                (String::from("field2"), GffFieldValue::Byte(2)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();
        /* header indicates 1 struct stored */
        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 2);
        assert_field_indice_count(&packer, 2);
        assert_label_count(&packer, 2);
    }

    #[test]
    fn test_03_pack_all_simple_fields() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Byte(1)),
                (String::from("field2"), GffFieldValue::Char(2)),
                (String::from("field3"), GffFieldValue::Word(3)),
                (String::from("field4"), GffFieldValue::Short(4)),
                (String::from("field5"), GffFieldValue::DWord(5)),
                (String::from("field6"), GffFieldValue::Int(6)),
                (String::from("field7"), GffFieldValue::Float(7.7)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();
        /* header indicates 1 struct stored */
        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 7);
        assert_field_indice_count(&packer, 7);
        assert_label_count(&packer, 7);
    }

    #[test]
    fn test_04_pack_all_8_byte_fields() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::DWord64(1)),
                (String::from("field2"), GffFieldValue::Int64(2)),
                (String::from("field3"), GffFieldValue::Double(3.3)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();
        /* header indicates 1 struct stored */
        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 3);
        assert_field_indice_count(&packer, 3);
        assert_label_count(&packer, 3);
        assert_field_data_count(&packer, 8 * 3);
    }

    #[test]
    fn test_05_pack_simple_string() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"),
                 GffFieldValue::CExoString(String::from("test"))),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 1);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 1);
        assert_field_data_count(&packer, 4 + 4);
        assert_eq!(
            packer.data.field_data,
            vec![4u8, 0, 0, 0,
                't' as u8, 'e' as u8, 's' as u8, 't' as u8]
        );
    }

    #[test]
    fn test_06_pack_resref() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"),
                 GffFieldValue::CResRef(String::from("TeSt"))),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 1);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 1);
        assert_field_data_count(&packer, 1 + 4);
        assert_eq!(
            packer.data.field_data,
            vec![4u8, 't' as u8, 'e' as u8, 's' as u8, 't' as u8]
        );
    }

    #[test]
    fn test_07_pack_locstr() {
        let langs = HashMap::from([
            ((GffLang::English, GffGender::Male), String::from("Hello")),
            ((GffLang::French, GffGender::Male), String::from("Salut")),
        ]);
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"),
                GffFieldValue::CExoLocString(0x1234, langs))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 1);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 1);
        assert_field_data_count(&packer, 12 + (8 + 5) + (8 + 5));
    }

    #[test]
    fn test_08_pack_void() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"),
                GffFieldValue::Void(b"test".to_vec()))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 1);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 1);
        assert_field_data_count(&packer, 4 + 4);
        assert_eq!(packer.data.field_data,
            vec![0x04, 0x00, 0x00, 0x00, 't' as u8, 'e' as u8, 's' as u8, 't' as u8]);
    }

    #[test]
    fn test_09_pack_sub_struct() {
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("subfield1"), GffFieldValue::Byte(1))
            ]),
        };
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Struct(input))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 2);
        assert_field_count(&packer, 2);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 2);
        assert_field_data_count(&packer, 0);
    }
    #[test]
    fn test_10_pack_list() {
        let sub1 = GffStruct {
            fields: HashMap::from([
                (String::from("subfield1"), GffFieldValue::Byte(1))
            ]),
        };
        let sub2 = GffStruct {
            fields: HashMap::from([
                (String::from("subfield2"), GffFieldValue::Byte(2))
            ]),
        };
        let input = GffStruct {
            fields: HashMap::from([
                (String::from("field1"),
                GffFieldValue::List(vec![sub1, sub2]))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 3);
        assert_field_count(&packer, 3);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 3);
        assert_field_data_count(&packer, 0);
        assert_list_count(&packer, 4 * 3); // 1 u32 for size, 2 for structs
    }
}
