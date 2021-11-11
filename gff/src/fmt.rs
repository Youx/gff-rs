use crate::common::{
    GffStruct,
    GffFieldValue,
};

trait DisplayDepth {
    fn fmt_depth(&self, depth: u32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::fmt::Display for GffStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_depth(0, f)?;
        writeln!(f, "")
    }
}

fn indent(f: &mut std::fmt::Formatter<'_>, depth: u32) -> std::fmt::Result {
    for _ in 0..depth {
        write!(f, "    ")?;
    }
    Ok(())
}
impl DisplayDepth for GffStruct {
    fn fmt_depth(&self, depth: u32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for (label, value) in &self.fields {
            indent(f, depth)?;
            write!(f, "{}: ", label)?;
            value.fmt_depth(depth, f)?;
            writeln!(f, ",")?;
        }
        indent(f, depth)?;
        write!(f, "}}")
    }
}

impl DisplayDepth for GffFieldValue {
    fn fmt_depth(&self, depth: u32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GffFieldValue::Byte(val) => write!(f, "{}", val),
            GffFieldValue::Char(val) => write!(f, "{}", val),
            GffFieldValue::Word(val) => write!(f, "{}", val),
            GffFieldValue::Short(val) => write!(f, "{}", val),
            GffFieldValue::DWord(val) => write!(f, "{}", val),
            GffFieldValue::Int(val) => write!(f, "{}", val),
            GffFieldValue::DWord64(val) => write!(f, "{}", val),
            GffFieldValue::Int64(val) => write!(f, "{}", val),
            GffFieldValue::Float(val) => write!(f, "{}", val),
            GffFieldValue::Double(val) => write!(f, "{}", val),
            GffFieldValue::Invalid => write!(f, "<invalid>"),
            GffFieldValue::CExoString(val) => write!(f, "\"{}\"", val),
            GffFieldValue::Struct(val) => val.fmt_depth(depth, f),
            GffFieldValue::List(val) => {
                write!(f, "[ ")?;
                for st in val {
                    st.fmt_depth(depth, f)?;
                    write!(f, ", ")?;
                }
                write!(f, "]")
            },
            GffFieldValue::CExoLocString(tlk_ref, val) => {
                if *tlk_ref == 0xFFFFFFFF {
                    writeln!(f, "{{")?;
                } else {
                    writeln!(f, "[{}]{{", tlk_ref)?;
                }
                for (lang, s) in val {
                    indent(f, depth + 1)?;
                    writeln!(f, "{:?}: \"{}\",", lang, s)?;
                }
                indent(f, depth)?;
                write!(f, "}}")
            },
            GffFieldValue::Void(val) => {
                let slice : &[u8] = &val;
                write!(f, "DATA{:.10X?}", slice)
            },
            GffFieldValue::CResRef(val) => write!(f, "Ref<{}>", val),
        }
    }
}
