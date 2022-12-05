//! Protocol Buffers inspection.

use anyhow::ensure;
use prost::bytes::Buf;
use prost::encoding::{decode_key, decode_varint, WireType};
use serde::Serialize;

use crate::prelude::*;

/// Represents a tagged value.
#[derive(Debug, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub enum Value {
    VarInt { as_u64: u64 },
    Fixed32 { as_u32: u32 },
    Fixed64 { as_u64: u64 },
    Message(Box<DynamicMessage>),
    Blob(String),
}

/// Dynamic (schemaless) Protocol Buffers message.
#[derive(Default, Debug, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct DynamicMessage(pub Vec<(u32, Value)>);

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
                    Some(Value::VarInt { as_u64 })
                }
                WireType::SixtyFourBit => {
                    ensure!(buffer.remaining() >= 8);
                    let as_u64 = buffer.get_u64_le();
                    Some(Value::Fixed64 { as_u64 })
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
                    let as_u32 = buffer.get_u32_le();
                    Some(Value::Fixed32 { as_u32 })
                }
            };
            if let Some(value) = value {
                this.0.push((tag, value));
            }
        }

        this.0.sort_unstable();
        Ok(this)
    }
}
