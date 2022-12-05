//! Protocol Buffers inspection.

use anyhow::ensure;
use prost::bytes::Buf;
use prost::encoding::{decode_key, decode_varint, WireType};
use serde::Serialize;

use crate::prelude::*;

/// Represents a tagged value.
#[derive(Debug, Serialize)]
pub enum Value {
    VarInt {
        as_u64: u64,
        as_i64: i64,
    },
    Fixed32 {
        as_u32: u32,
        as_i32: i32,
        as_f32: f32,
    },
    Fixed64 {
        as_u64: u64,
        as_i64: i64,
        as_f64: f64,
    },
    Message(Box<DynamicMessage>),
    Blob(String),
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub tag: u32,
    pub value: Value,
}

/// Dynamic (schemaless) Protocol Buffers message.
#[derive(Default, Debug, Serialize)]
pub struct DynamicMessage(pub Vec<Entry>);

impl DynamicMessage {
    /// Decodes the message from the buffer.
    ///
    /// Because `prost` nor `prost-reflect` don't provide this kind of functionality,
    /// I've implemented it manually based on the low-level `prost` functions.
    pub fn decode(buffer: &mut impl Buf) -> Result<Self> {
        let mut this = DynamicMessage::default();

        while buffer.remaining() != 0 {
            let (tag, wire_type) = decode_key(buffer)?;
            let value = match wire_type {
                WireType::Varint => {
                    let as_u64 = decode_varint(buffer)?;
                    Some(Value::VarInt {
                        as_u64,
                        as_i64: from_uint64(as_u64),
                    })
                }
                WireType::SixtyFourBit => {
                    ensure!(buffer.remaining() >= 8);
                    let bytes = buffer.copy_to_bytes(8);
                    Some(Value::Fixed64 {
                        as_u64: (&mut bytes.as_ref()).get_u64_le(),
                        as_i64: (&mut bytes.as_ref()).get_i64_le(),
                        as_f64: (&mut bytes.as_ref()).get_f64_le(),
                    })
                }
                WireType::LengthDelimited => {
                    let length = decode_varint(buffer)? as usize;
                    ensure!(buffer.remaining() >= length);
                    let blob = buffer.copy_to_bytes(length);
                    match DynamicMessage::decode(&mut blob.as_ref()) {
                        Ok(message) => Some(Value::Message(Box::new(message))),
                        Err(_) => Some(Value::Blob(String::from_utf8_lossy(&blob).into_owned())),
                    }
                }
                WireType::StartGroup => {
                    // Ignore group validation.
                    None
                }
                WireType::EndGroup => {
                    // Ignore group validation.
                    None
                }
                WireType::ThirtyTwoBit => {
                    ensure!(buffer.remaining() >= 4);
                    let bytes = buffer.copy_to_bytes(4);
                    Some(Value::Fixed32 {
                        as_u32: (&mut bytes.as_ref()).get_u32_le(),
                        as_i32: (&mut bytes.as_ref()).get_i32_le(),
                        as_f32: (&mut bytes.as_ref()).get_f32_le(),
                    })
                }
            };
            if let Some(value) = value {
                this.0.push(Entry { tag, value });
            }
        }

        this.0.sort_unstable_by_key(|entry| entry.tag);
        Ok(this)
    }
}

/// Stolen from `prost::encoding`.
const fn from_uint64(value: u64) -> i64 {
    ((value >> 1) as i64) ^ (-((value & 1) as i64))
}
