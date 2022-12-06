use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Value<'a> {
    VarInt {
        unsigned: u64,
        signed: i64,
    },
    Fixed64 {
        #[serde(rename = "u64")]
        as_u64: u64,
        #[serde(rename = "i64")]
        as_i64: i64,
        #[serde(rename = "f64")]
        as_f64: f64,
    },
    Fixed32 {
        #[serde(rename = "u32")]
        as_u32: u32,
        #[serde(rename = "i32")]
        as_i32: i32,
        #[serde(rename = "f32")]
        as_f32: f32,
    },
    Message(Box<Message<'a>>),
    Bytes {
        #[serde(serialize_with = "hex::serde::serialize")]
        raw: &'a [u8],
        #[serde(skip_serializing_if = "Option::is_none", rename = "str")]
        as_str: Option<&'a str>,
    },
}

impl<'a> Value<'a> {
    pub const fn varint(value: u64) -> Self {
        Self::VarInt {
            unsigned: value,
            signed: Self::zigzag(value),
        }
    }

    pub fn fixed64(value: u64) -> Self {
        Self::Fixed64 {
            as_u64: value,
            as_i64: i64::from_le_bytes(value.to_le_bytes()),
            as_f64: f64::from_le_bytes(value.to_le_bytes()),
        }
    }

    pub fn fixed32(value: u32) -> Self {
        Self::Fixed32 {
            as_u32: value,
            as_i32: i32::from_le_bytes(value.to_le_bytes()),
            as_f32: f32::from_le_bytes(value.to_le_bytes()),
        }
    }

    pub fn bytes(value: &'a [u8]) -> Self {
        Self::Bytes {
            raw: value,
            as_str: std::str::from_utf8(value).ok(),
        }
    }

    const fn zigzag(value: u64) -> i64 {
        ((value >> 1) as i64) ^ (-((value & 1) as i64))
    }
}

#[derive(Serialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[serde(into = "String")]
pub struct Tag(pub u32);

impl From<Tag> for String {
    fn from(tag: Tag) -> Self {
        tag.0.to_string()
    }
}

#[derive(Serialize)]
pub struct Message<'a>(pub BTreeMap<Tag, Vec<Value<'a>>>);
