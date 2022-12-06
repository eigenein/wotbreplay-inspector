//! Protocol Buffers inspection.

mod inspector;
mod visit;

use anyhow::{ensure, Context};
use prost::bytes::Buf;
use prost::encoding::{decode_key, decode_varint, WireType};

pub use self::inspector::*;
use self::visit::Visit;
use crate::prelude::*;

/// Inspects the message in the buffer.
///
/// Because `prost` nor `prost-reflect` don't provide this kind of functionality,
/// I've implemented it manually based on the low-level `prost` functions.
pub fn inspect(buffer: &[u8], visit: &mut impl Visit) -> Result {
    let mut buffer = buffer;
    while buffer.remaining() != 0 {
        let (tag, wire_type) = decode_key(&mut buffer).context("failed to decode the field key")?;
        match wire_type {
            WireType::Varint => {
                let value = decode_varint(&mut buffer).with_context(|| {
                    format!("failed to decode the varint value for tag {}", tag)
                })?;
                visit.varint(tag, value);
            }
            WireType::ThirtyTwoBit => {
                visit.fixed32(tag, buffer.get_u32_le());
            }
            WireType::SixtyFourBit => {
                visit.fixed64(tag, buffer.get_u64_le());
            }
            WireType::StartGroup => {
                visit.start_group(tag);
            }
            WireType::EndGroup => {
                visit.end_group(tag);
            }
            WireType::LengthDelimited => {
                let length = decode_varint(&mut buffer)
                    .with_context(|| format!("failed to decode the length for tag {}", tag))?
                    as usize;
                let inner_buffer = &buffer[..length];
                match validate_message(inner_buffer) {
                    Ok(_) => {
                        visit.start_message(tag);
                        inspect(inner_buffer, visit).with_context(|| {
                            format!("failed to inspect the message for tag {}", tag)
                        })?;
                        visit.end_message(tag);
                    }
                    Err(_) => match std::str::from_utf8(inner_buffer) {
                        Ok(value) => {
                            visit.string(tag, value)?;
                        }
                        Err(_) => {
                            visit.bytes(tag, inner_buffer)?;
                        }
                    },
                }
                buffer.advance(length);
            }
        };
    }
    Ok(())
}

/// Validates correctness of the buffer prior to calling the visitor.
fn validate_message(buffer: &[u8]) -> Result {
    let mut buffer = buffer;
    while buffer.remaining() != 0 {
        let (_tag, wire_type) = decode_key(&mut buffer)?;
        match wire_type {
            WireType::Varint => {
                decode_varint(&mut buffer)?;
            }
            WireType::ThirtyTwoBit => {
                ensure!(buffer.remaining() >= 4);
                buffer.advance(4);
            }
            WireType::SixtyFourBit => {
                ensure!(buffer.remaining() >= 8);
                buffer.advance(8);
            }
            WireType::LengthDelimited => {
                let length = decode_varint(&mut buffer)? as usize;
                ensure!(buffer.remaining() >= length);
                buffer.advance(length);
            }
            WireType::StartGroup => {}
            WireType::EndGroup => {}
        }
    }
    Ok(())
}
