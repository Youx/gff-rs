use std::collections::HashMap;
use std::io::Write;
use std::borrow::Cow;

use crate::common::{
    GffHeader,
    GffStruct,
    GffFieldValue,
    GffLang,
    GffGender,
    EncodingFn,
};

pub struct PackData {
    pub header: GffHeader,
    pub structs: Vec<u8>,
    pub fields: Vec<u8>,
    pub labels: Vec<u8>,
    pub field_data: Vec<u8>,
    pub field_indices: Vec<u8>,
    pub list_indices: Vec<u8>,
}

pub struct Packer<'enc, W: std::io::Write> {
    pub writer: std::io::BufWriter<W>,
    labels: HashMap<String, u32>,
    pub data: PackData,
    encodings: &'enc EncodingFn,
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

impl <'enc, 'input, W: std::io::Write> Packer<'enc, W> {
    pub fn new(writer: W, encodings: &'enc EncodingFn) -> Packer<'enc, W> {
        Packer {
            writer: std::io::BufWriter::new(writer),
            labels: HashMap::new(),
            data: PackData::new(),
            encodings: encodings,
        }
    }

    /// Write offsets into header.
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

    fn write(&mut self) -> Result<(), std::io::Error> {
        /* write header */
        self.writer.write(&self.data.header.gff_type)?;
        self.writer.write(&self.data.header.version)?;
        self.writer.write(&self.data.header.structs.0.to_le_bytes())?;
        self.writer.write(&self.data.header.structs.1.to_le_bytes())?;
        self.writer.write(&self.data.header.fields.0.to_le_bytes())?;
        self.writer.write(&self.data.header.fields.1.to_le_bytes())?;
        self.writer.write(&self.data.header.labels.0.to_le_bytes())?;
        self.writer.write(&self.data.header.labels.1.to_le_bytes())?;
        self.writer.write(&self.data.header.field_data.0.to_le_bytes())?;
        self.writer.write(&self.data.header.field_data.1.to_le_bytes())?;
        self.writer.write(&self.data.header.field_indices.0.to_le_bytes())?;
        self.writer.write(&self.data.header.field_indices.1.to_le_bytes())?;
        self.writer.write(&self.data.header.list_indices.0.to_le_bytes())?;
        self.writer.write(&self.data.header.list_indices.1.to_le_bytes())?;

        self.writer.write(&self.data.structs)?;
        self.writer.write(&self.data.fields)?;
        self.writer.write(&self.data.labels)?;
        self.writer.write(&self.data.field_data)?;
        self.writer.write(&self.data.field_indices)?;
        self.writer.write(&self.data.list_indices)?;
        Ok(())
    }

    /* {{{ Pack functions */

    /// Pack a GffStruct.
    ///
    /// This is used as the entry point of data packing.
    ///
    pub fn pack(&mut self, input: &'input GffStruct)
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
        self.write().map_err(|_e| "failed to write packed data")?;

        Ok(())
    }

    ///  Pack a GffStruct.
    ///
    /// This packs all basic field data.
    /// Structs/Lists of structs will be pushed in a vec and
    /// packed afterwards.
    ///
    fn pack_struct(&mut self, input: &'input GffStruct,
        structs: &mut Vec<&'input GffStruct>, current_st_idx: &mut u32)
        -> Result<(), &'static str>
    {
        /* write struct type */
        self.data.structs.extend_from_slice(&input.st_type.to_le_bytes());
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

    ///  Pack a field label into the labels block.
    ///
    /// A field must be <= 16 chars, and will be padded with 0 if shorter.
    ///
    pub fn pack_label(&mut self, label: &String)
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

    /// Pack a struct field name and associated value.
    fn pack_field(&mut self, field_name: &String, field_value: &'input GffFieldValue,
        structs: &mut Vec<&'input GffStruct>, current_st_idx: &mut u32)
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
                let encodings = self.encodings;
                let encoding = encodings(None).unwrap();

                self.pack_data_offset(10, label_idx);

                let (str_data, _, _) = encoding.encode(s);
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
                let encodings = self.encodings;

                let val_encoded: Vec<(GffLang, GffGender, Cow<'_, [u8]>)> =
                    val.into_iter().map(|((lang, gender), s)| {
                        let encoding = encodings(Some(*lang as u32)).unwrap();
                        let (s_vec, _, _) = encoding.encode(s);
                        // gender-lang + length + string
                        total_len += 8 + s_vec.len() as u32;
                        (*lang, *gender, s_vec)
                }).collect();

                // total data size
                self.pack_data_u32(total_len);
                // string ref
                self.pack_data_u32(*str_ref);
                // string count
                self.pack_data_u32(val.len() as u32);

                for (lang, gender, s) in val_encoded {
                    let gender = gender as u32;
                    let lang = lang as u32;
                    // gender-lang
                    self.pack_data_u32(gender + 2 * lang);
                    // length
                    self.pack_data_u32(s.len() as u32);
                    // string
                    self.pack_data_slice(&s);

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

    /// Pack a field type, field label, and 1 byte of data into the fields block.
    ///
    /// The 1 byte of data will be padded with 3 bytes of zeros.
    ///
    fn pack_val_1(&mut self, ftype: u32, label_idx: u32, val: u8) {
        self.data.fields.extend_from_slice(&ftype.to_le_bytes());
        self.data.fields.extend_from_slice(&label_idx.to_le_bytes());
        self.data.fields.push(val);
        self.data.fields.extend_from_slice(&[0u8; 3]);
    }

    /// Pack a field type, field label, and 2 bytes of data into the fields block.
    ///
    /// The 2 bytes of data will be padded with 2 bytes of zeros.
    ///
    fn pack_val_2(&mut self, ftype: u32, label_idx: u32, val: &[u8; 2]) {
        self.data.fields.extend_from_slice(&ftype.to_le_bytes());
        self.data.fields.extend_from_slice(&label_idx.to_le_bytes());
        self.data.fields.extend_from_slice(val);
        self.data.fields.extend_from_slice(&[0u8; 2]);
    }

    /// Pack a field type, field label, and 4 bytes of data into the fields block.
    pub fn pack_val_4(&mut self, ftype: u32, label_idx: u32, val: &[u8; 4]) {
        self.data.fields.extend_from_slice(&ftype.to_le_bytes());
        self.data.fields.extend_from_slice(&label_idx.to_le_bytes());
        self.data.fields.extend_from_slice(val);
    }

    /// Pack a field type, field label, and data_offset into the fields block.
    fn pack_data_offset(&mut self, ftype: u32, label_idx: u32) {
        self.pack_val_4(ftype, label_idx,
            &(self.data.field_data.len() as u32).to_le_bytes()
        );
    }

    /// Pack a field type, field label, and 8 bytes of data.
    ///
    /// The field type, field label, and field_data offset will be stored
    /// in the fields block, and the 8 bytes of data will be packed into
    /// the field_data block.
    ///
    fn pack_val_8(&mut self, ftype: u32, label_idx: u32, val: &[u8; 8]) {
        self.pack_data_offset(ftype, label_idx);
        self.data.field_data.extend_from_slice(val);
        self.data.header.field_data.1 += 8;
    }

    /// Pack an u32 into the field_data block.
    fn pack_data_u32(&mut self, val: u32) {
        self.data.field_data.extend_from_slice(&val.to_le_bytes());
        self.data.header.field_data.1 += 4;
    }

    /// Pack an arbitrary byte array into the field_data block.
    fn pack_data_slice(&mut self, val: &[u8]) {
        self.data.field_data.extend_from_slice(val);
        self.data.header.field_data.1 += val.len() as u32;
    }

    /// Pack an u32 into the list_indices block.
    fn pack_list_u32(&mut self, val: u32) {
        self.data.list_indices.extend_from_slice(&val.to_le_bytes());
        self.data.header.list_indices.1 += 4;
    }

    /* }}} */
}

/* {{{ PackField implementations. */

/* Trait to pack field */
pub trait PackField<'a, W: std::io::Write> {
    fn pack_field(&'a self, label: String, packer: &mut Packer<W>,
        structs: &mut Vec<&'a dyn PackStruct<W>>, st_idx: &mut u32) -> ();
}

/* Trait to pack struct */
pub trait PackStruct<'a, W: std::io::Write> {
    fn pack(&'a self, data: &mut Packer<W>, structs: &mut Vec<&'a dyn PackStruct<W>>,
        st_idx: &mut u32)
        -> ();
}

macro_rules! pack_field_1 {
    ( $type:ident, $type_id:literal ) => {
        impl<'a, W: std::io::Write> PackField<'a, W> for $type {
            fn pack_field(&'a self, label: String, packer: &mut Packer<W>,
                _structs: &mut Vec<&'a dyn PackStruct<W>>, _st_idx: &mut u32)
                -> ()
            {
                let label_idx = packer.pack_label(&label).unwrap();
                packer.pack_val_1(1, label_idx, *self as u8);
            }
        }
    }
}

pack_field_1!(u8, 0);
pack_field_1!(i8, 1);

macro_rules! pack_field_n {
    ( $type:ident, $pack_fn:ident, $type_id:literal ) => {
        impl<'a, W: std::io::Write> PackField<'a, W> for $type {
            fn pack_field(&'a self, label: String, packer: &mut Packer<W>,
                _structs: &mut Vec<&'a dyn PackStruct<W>>, _st_idx: &mut u32)
                -> ()
            {
                let label_idx = packer.pack_label(&label).unwrap();
                packer.$pack_fn(1, label_idx, &self.to_le_bytes());
            }
        }
    }
}

pack_field_n!(u16, pack_val_2, 2);
pack_field_n!(i16, pack_val_2, 3);
pack_field_n!(u32, pack_val_4, 4);
pack_field_n!(i32, pack_val_4, 5);
pack_field_n!(u64, pack_val_8, 6);
pack_field_n!(i64, pack_val_8, 7);
pack_field_n!(f32, pack_val_4, 8);
pack_field_n!(f64, pack_val_8, 9);

impl<'a, W: std::io::Write> PackField<'a, W> for String {
    fn pack_field(&'a self, label: String, packer: &mut Packer<W>,
        _structs: &mut Vec<&'a dyn PackStruct<W>>, _st_idx: &mut u32)
        -> ()
    {
        let encodings = packer.encodings;
        let encoding = encodings(None).unwrap();
        let label_idx = packer.pack_label(&label).unwrap();

        packer.pack_data_offset(10, label_idx);

        let (str_data, _, _) = encoding.encode(self);
        packer.pack_data_u32(str_data.len() as u32);
        packer.pack_data_slice(&str_data);
    }
}

/* }}} */

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::packer::Packer;
    use crate::common::{
        GffStruct,
        GffFieldValue,
        GffLang,
        GffGender,
        Encodings,
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
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Byte(1)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 1);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 1);
    }

    #[test]
    fn test_02_pack_2_simple_fields() {
        let input = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Byte(1)),
                (String::from("field2"), GffFieldValue::Byte(2)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0xFFFFFFFF,
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
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::DWord64(1)),
                (String::from("field2"), GffFieldValue::Int64(2)),
                (String::from("field3"), GffFieldValue::Double(3.3)),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"),
                 GffFieldValue::CExoString(String::from("test"))),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"),
                 GffFieldValue::CResRef(String::from("TeSt"))),
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"),
                GffFieldValue::CExoLocString(0x1234, langs))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"),
                GffFieldValue::Void(b"test".to_vec()))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0x55555555,
            fields: HashMap::from([
                (String::from("subfield1"), GffFieldValue::Byte(1))
            ]),
        };
        let input = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"), GffFieldValue::Struct(input))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
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
            st_type: 0x55555555,
            fields: HashMap::from([
                (String::from("subfield1"), GffFieldValue::Byte(1))
            ]),
        };
        let sub2 = GffStruct {
            st_type: 0xAAAAAAAA,
            fields: HashMap::from([
                (String::from("subfield2"), GffFieldValue::Byte(2))
            ]),
        };
        let input = GffStruct {
            st_type: 0xFFFFFFFF,
            fields: HashMap::from([
                (String::from("field1"),
                GffFieldValue::List(vec![sub1, sub2]))
            ]),
        };
        let output = Vec::new();
        let mut packer = Packer::new(output, &*Encodings::NeverwinterNights);
        packer.pack(&input).unwrap();

        assert_struct_count(&packer, 3);
        assert_field_count(&packer, 3);
        assert_field_indice_count(&packer, 0);
        assert_label_count(&packer, 3);
        assert_field_data_count(&packer, 0);
        assert_list_count(&packer, 4 * 3); // 1 u32 for size, 2 for structs
    }
}
