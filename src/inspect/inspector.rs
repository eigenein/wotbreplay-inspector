use std::io::{stdout, Write};

use crate::inspect::visit::Visit;
use crate::prelude::*;

#[derive(Default)]
pub struct Inspector {
    path: Vec<u32>,
}

impl Visit for Inspector {
    fn varint(&self, tag: u32, value: u64) {
        let _ = writeln!(stdout(), "# {}: varint", tag);
        let _ = writeln!(
            stdout(),
            "{} = {{ u64 = {}, i64 = {} }}",
            tag,
            value,
            Inspector::from_uint64(value)
        );
    }

    fn fixed32(&self, tag: u32, value: u32) {
        let _ = writeln!(stdout(), "# {}: fixed32", tag);
        let _ = writeln!(
            stdout(),
            "{} = {{ u32 = {}, i32 = {}, f32 = {} }}",
            tag,
            value,
            i32::from_le_bytes(value.to_le_bytes()),
            f32::from_le_bytes(value.to_le_bytes()),
        );
    }

    fn fixed64(&self, tag: u32, value: u64) {
        let _ = writeln!(stdout(), "# {}: fixed64", tag);
        let _ = writeln!(
            stdout(),
            "{} = {{ u64 = {}, i64 = {}, f64 = {} }}",
            tag,
            value,
            i64::from_le_bytes(value.to_le_bytes()),
            f64::from_le_bytes(value.to_le_bytes()),
        );
    }

    fn string(&self, tag: u32, value: &str) -> Result {
        let _ = writeln!(stdout(), "# {}: string", tag);
        let _ = writeln!(stdout(), "{} = {}", tag, toml::to_string(value)?);
        Ok(())
    }

    fn bytes(&self, tag: u32, value: &[u8]) -> Result {
        let _ = writeln!(stdout(), "# {}: bytes", tag);
        let _ = write!(stdout(), "{} = \"", tag);
        for byte in value {
            let _ = write!(stdout(), "\\x{:02x}", byte);
        }
        let _ = writeln!(stdout(), "\" # bytes({})", value.len());
        Ok(())
    }

    fn start_group(&self, tag: u32) {
        let _ = writeln!(stdout(), "# start group #{}", tag);
    }

    fn end_group(&self, tag: u32) {
        let _ = writeln!(stdout(), "# end group #{}", tag);
    }

    fn start_message(&mut self, tag: u32) {
        let _ = writeln!(stdout());
        let _ = writeln!(stdout(), "# start message #{}", tag);
        self.path.push(tag);
        self.write_header();
    }

    fn end_message(&mut self, tag: u32) {
        self.path.pop();
        let _ = writeln!(stdout(), "# end message #{}", tag);
        let _ = writeln!(stdout());
        self.write_header();
    }
}

impl Inspector {
    fn write_header(&self) {
        if !self.path.is_empty() {
            let _ = write!(stdout(), "[");
            for (i, tag) in self.path.iter().copied().enumerate() {
                if i != 0 {
                    let _ = write!(stdout(), ".");
                }
                let _ = write!(stdout(), "{}", tag);
            }
            let _ = writeln!(stdout(), "]");
        }
    }

    /// Converts unsigned integer using the
    /// [ZigZag encoding](https://en.wikipedia.org/wiki/Variable-length_quantity#Zigzag_encoding).
    ///
    /// Stolen from `prost::encoding`.
    const fn from_uint64(value: u64) -> i64 {
        ((value >> 1) as i64) ^ (-((value & 1) as i64))
    }
}
