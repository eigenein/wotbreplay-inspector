use crate::prelude::*;

/// Used to visit tags in a message.
pub trait Visit {
    fn varint(&self, tag: u32, value: u64);
    fn fixed32(&self, tag: u32, value: u32);
    fn fixed64(&self, tag: u32, value: u64);
    fn string(&self, tag: u32, value: &str) -> Result;
    fn bytes(&self, tag: u32, value: &[u8]) -> Result;
    fn start_group(&self, tag: u32);
    fn end_group(&self, tag: u32);
    fn start_message(&mut self, tag: u32);
    fn end_message(&mut self, tag: u32);
}
