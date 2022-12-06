//! Protocol Buffers inspection.

mod models;
mod visit;

use std::collections::BTreeMap;

use anyhow::{ensure, Context};
use prost::bytes::Buf;
use prost::encoding::{decode_key, decode_varint, WireType};

pub use self::models::*;
use crate::prelude::*;

/// Inspects the message in the buffer.
///
/// Because `prost` nor `prost-reflect` don't provide this kind of functionality,
/// I've implemented it manually based on the low-level `prost` functions.
pub fn inspect(buffer: &[u8]) -> Result<Message> {
    let mut fields: BTreeMap<Tag, Vec<Value>> = BTreeMap::new();
    let mut buffer = buffer;

    while buffer.remaining() != 0 {
        let (tag, wire_type) = decode_key(&mut buffer).context("failed to decode the field key")?;
        let tag = Tag(tag);
        let value = match wire_type {
            WireType::ThirtyTwoBit => Some(Value::fixed32(buffer.get_u32_le())),
            WireType::SixtyFourBit => Some(Value::fixed64(buffer.get_u64_le())),
            WireType::Varint => {
                let value = decode_varint(&mut buffer).with_context(|| {
                    format!("failed to decode the varint value for tag {}", tag.0)
                })?;
                Some(Value::varint(value))
            }
            WireType::LengthDelimited => {
                let length = decode_varint(&mut buffer)
                    .with_context(|| format!("failed to decode the length for tag {}", tag.0))?
                    as usize;
                let inner_buffer = &buffer[..length];
                let value = match validate_message(inner_buffer) {
                    Ok(_) => {
                        let message = inspect(inner_buffer).with_context(|| {
                            format!("failed to inspect the message for tag {}", tag.0)
                        })?;
                        Some(Value::Message(Box::new(message)))
                    }
                    Err(_) => Some(Value::bytes(inner_buffer)),
                };
                buffer.advance(length);
                value
            }
            WireType::StartGroup => None,
            WireType::EndGroup => None,
        };
        if let Some(value) = value {
            fields.entry(tag).or_default().push(value);
        }
    }

    Ok(Message(fields))
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
