pub mod common;
pub mod deserialize;
pub mod parser;
pub mod packer;


#[cfg(test)]
mod tests {
    use crate::parser::GffParser;
    use crate::packer::Packer;
    use crate::common::GffStruct;

    fn test_pack_unpack(input: &GffStruct) {
        let output = Vec::new();
        let mut packer = Packer::new(output);

        packer.pack(&input).unwrap();
        let data = packer.writer.into_inner().unwrap();
        let res = GffParser::parse(data, "").unwrap();
        assert_eq!(res, *input);
    }
}
