use std::collections::HashMap;
use crate::common::{
    GffHeader,
    GffStruct,
    GffFieldValue,
};

struct PackData<'a> {
    header: GffHeader<'a>,
    structs: Vec<u8>,
    fields: Vec<u8>,
    labels: Vec<u8>,
    field_data: Vec<u8>,
    field_indices: Vec<u8>,
    list_indices: Vec<u8>,
}

pub struct Packer<'a, W: std::io::Write> {
    writer: std::io::BufWriter<W>,
    labels: HashMap<String, u32>,
    data: PackData<'a>,
}

impl PackData<'_> {
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

impl <'a, W: std::io::Write> Packer<'a, W> {
    pub fn new(writer: W) -> Packer<'a, W> {
        Packer {
            writer: std::io::BufWriter::new(writer),
            labels: HashMap::new(),
            data: PackData::new(),
        }
    }

    /* {{{ Pack functions */
    pub fn pack(&mut self, input: &GffStruct) {
        self.pack_struct(input).unwrap();
    }

    fn pack_struct(&mut self, input: &GffStruct)
    -> Result<(), &'static str>
    {
        /* write struct type */
        self.data.structs.extend_from_slice(&0u32.to_le_bytes());
        match input.fields.len() {
            0 => return Err("struct must contain at least one field"),
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
            let field_id = self.pack_field(&field, &value)?;

            field_indices.push(field_id);
        }

        if field_indices.len() > 1 {
            /* write fields indices into field_indices array */
            for field_indice in field_indices {
                self.data.field_indices.extend_from_slice(
                    &(field_indice as u32).to_le_bytes()
                );
                self.data.header.field_indices.1 += 1;
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

    fn pack_field(&mut self, field_name: &String,
                  field_value: &GffFieldValue)
        -> Result<u32, &'static str>
    {
        let label_idx = self.pack_label(field_name)?;

        self.data.header.fields.1 += 1;

        match field_value {
            GffFieldValue::Byte(val) => {
                self.data.fields.extend_from_slice(
                    &(0 as u32).to_le_bytes()
                );
                self.data.fields.extend_from_slice(
                    &(label_idx as u32).to_le_bytes()
                );
                self.data.fields.push(*val);
                self.data.fields.extend_from_slice(
                    &[0u8; 3]
                );
                Ok(self.data.header.fields.1)
            }
            _ => Err("Not handled yet")
        }
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
        assert_eq!(packer.data.header.field_indices.1, fi_count as u32);
        // 1 DWORDS per field
        assert_eq!(packer.data.field_indices.len(), fi_count * 4);
    }
    fn assert_label_count(packer: &Packer<Vec<u8>>, l_count: usize) {
        assert_eq!(packer.data.header.labels.1, l_count as u32);
        // 16 bytes per label
        assert_eq!(packer.data.labels.len(), l_count * 16);
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
        packer.pack(&input);

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
        packer.pack(&input);
        /* header indicates 1 struct stored */
        assert_struct_count(&packer, 1);
        assert_field_count(&packer, 2);
        assert_field_indice_count(&packer, 2);
        assert_label_count(&packer, 2);
    }
}
