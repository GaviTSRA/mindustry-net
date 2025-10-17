use crate::packet::{Packet, PacketError, parse_regular_packet};
use crate::type_io::Reader;
use std::collections::HashMap;

pub struct StreamBuilder {
    pub id: u32,
    total: u32,
    stream_type: u8,
    data: Vec<u8>,
}

impl StreamBuilder {
    pub fn new(id: u32, stream_type: u8, total: u32) -> StreamBuilder {
        StreamBuilder {
            id,
            total,
            stream_type,
            data: vec![],
        }
    }

    pub fn add(&mut self, data: Vec<u8>) {
        self.data.extend(data);
    }

    pub fn is_done(&self) -> bool {
        self.data.len() >= self.total as usize
    }

    pub fn build(
        self,
        content_map: &Option<HashMap<String, Vec<String>>>,
    ) -> Result<Packet, PacketError> {
        let reader = Reader::new(self.data);
        parse_regular_packet(self.stream_type, reader, content_map)
    }
}
