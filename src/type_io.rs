use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::HashMap;

use crate::arc_types::Point2;

pub struct Reader {
    buf: Vec<u8>,
    pos: usize,
}
impl Reader {
    pub fn new(buf: Vec<u8>) -> Reader {
        Reader { buf, pos: 0 }
    }

    pub fn byte(&mut self) -> u8 {
        let b = self.buf[self.pos];
        self.pos += 1;
        b
    }

    pub fn bool(&mut self) -> bool {
        self.byte() == 1
    }

    pub fn bytes(&mut self, n: usize) -> Vec<u8> {
        let end = (self.pos + n).min(self.buf.len());
        let bytes = self.buf[self.pos..end].to_vec();
        self.pos = end;
        bytes
    }

    pub fn unsigned_short(&mut self) -> u16 {
        let bytes = self.bytes(2);
        u16::from_be_bytes([bytes[0], bytes[1]])
    }

    pub fn short(&mut self) -> i16 {
        let bytes = self.bytes(2);
        i16::from_be_bytes([bytes[0], bytes[1]])
    }

    pub fn int(&mut self) -> u32 {
        let bytes = self.bytes(4);
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    pub fn long(&mut self) -> u64 {
        let bytes = self.bytes(8);
        u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    pub fn float(&mut self) -> f32 {
        let bytes = self.bytes(4);
        f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    pub fn double(&mut self) -> f64 {
        let bytes = self.bytes(8);
        f64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    pub fn read_remaining(&mut self) -> Vec<u8> {
        let bytes = self.buf[self.pos..].to_vec();
        self.pos = self.buf.len();
        bytes
    }
}

pub fn write_byte(buf: &mut Vec<u8>, value: u8) {
    buf.push(value);
}

pub fn write_bool(buf: &mut Vec<u8>, value: bool) {
    buf.push(if value { 1 } else { 0 });
}

pub fn write_short(buf: &mut Vec<u8>, short: i16) {
    buf.extend_from_slice(&short.to_be_bytes());
}

pub fn write_unsigned_short(buf: &mut Vec<u8>, short: u16) {
    buf.extend_from_slice(&short.to_be_bytes());
}

pub fn write_int(buf: &mut Vec<u8>, int: u32) {
    buf.extend_from_slice(&int.to_be_bytes());
}

pub fn write_long(buf: &mut Vec<u8>, long: u64) {
    buf.extend_from_slice(&long.to_be_bytes());
}

pub fn write_float(buf: &mut Vec<u8>, data: f32) {
    buf.extend_from_slice(&data.to_be_bytes());
}
pub fn write_double(buf: &mut Vec<u8>, data: f64) {
    buf.extend_from_slice(&data.to_be_bytes());
}

pub fn read_prefixed_string(reader: &mut Reader) -> Option<String> {
    if reader.remaining() == 0 {
        return None;
    }

    let has_string = reader.byte() != 0;
    if !has_string {
        return None;
    }

    if reader.remaining() < 2 {
        return None;
    }
    let length = reader.unsigned_short() as usize;

    if reader.remaining() < length {
        return None;
    }
    let string_bytes: Vec<u8> = reader.bytes(length);
    String::from_utf8(string_bytes).ok()
}

pub fn read_string(reader: &mut Reader) -> Option<String> {
    let length = reader.unsigned_short();
    if length == 0 {
        return None;
    }
    let string_bytes: Vec<u8> = reader.bytes(length as usize);
    String::from_utf8(string_bytes).ok()
}

pub fn write_string(buf: &mut Vec<u8>, string: &str) {
    if !string.is_empty() {
        buf.push(1);
        let encoded = string.as_bytes();
        let length = encoded.len();

        // Length should fit in 2 bytes
        assert!(length <= u16::MAX as usize, "String too long");

        buf.extend_from_slice(&(length as u16).to_be_bytes());
        buf.extend_from_slice(encoded);
    } else {
        buf.push(0);
    }
}

// TODO
#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Int(u32),
    Long(u64),
    Float(f32),
    String(Option<String>),
    // Content
    IntSequence(Vec<u32>),
    Point2(Point2),
    Point2Array(Vec<Point2>),
    // TechNode
    Boolean(bool),
    Double(f64),
    // Building
    // LAccess
    ByteArray(Vec<u8>),
    BooleanArray(Vec<bool>),
    // Unit
    Vec2Array(Vec<Vec2>),
    Vec2(Vec2),
    // Team
    // int[]
    // Object[]
    // UnitCommand
    Unknown,

    NotImplemented,
}

pub fn read_object_boxed(reader: &mut Reader, box_: bool) -> Object {
    read_object(reader)
}

// TODO
pub fn read_object(reader: &mut Reader) -> Object {
    let object_type = reader.byte();

    match object_type {
        0 => Object::Null,
        1 => Object::Int(reader.int()),
        2 => Object::Long(reader.long()),
        3 => Object::Float(reader.float()),
        4 => Object::String(read_prefixed_string(reader)),
        5 => {
            reader.byte();
            reader.short();
            Object::NotImplemented
        }
        6 => {
            let length = reader.short();
            let mut values = vec![];
            for _ in 0..length {
                values.push(reader.int());
            }
            Object::IntSequence(values)
        }
        7 => {
            let x = reader.int() as i16;
            let y = reader.int() as i16;
            Object::Point2(Point2 { x, y })
        }
        8 => {
            let length = reader.byte();
            let mut values = vec![];
            for _ in 0..length {
                values.push(Point2::unpack(reader.int()));
            }
            Object::Point2Array(values)
        }
        9 => {
            reader.byte();
            reader.short();
            Object::NotImplemented
        }
        10 => {
            let value = reader.bool();
            Object::Boolean(value)
        }
        11 => {
            let value = reader.double();
            Object::Double(value)
        }
        12 => {
            reader.int();
            Object::NotImplemented
        }
        13 => {
            reader.short();
            Object::NotImplemented
        }
        14 => {
            let length = reader.int();
            let mut values = vec![];
            for _ in 0..length {
                values.push(reader.byte());
            }
            Object::ByteArray(values)
        }
        //15 => {
        //    reader.byte();
        //    Object::NotImplemented
        //}
        16 => {
            let length = reader.short();
            let mut values = vec![];
            for _ in 0..length {
                values.push(reader.bool());
            }
            Object::BooleanArray(values)
        }
        17 => {
            reader.int();
            Object::NotImplemented
        }
        18 => {
            let length = reader.short();
            let mut values = vec![];
            for _ in 0..length {
                let x = reader.float();
                let y = reader.float();
                values.push(Vec2 { x, y });
            }
            Object::Vec2Array(values)
        }
        19 => {
            let x = reader.float();
            let y = reader.float();
            Object::Vec2(Vec2 { x, y })
        }
        20 => {
            reader.byte();
            Object::NotImplemented
        }
        21 => {
            let length = reader.short();
            let mut values = vec![];
            for _ in 0..length {
                values.push(reader.int());
            }
            Object::NotImplemented
        }
        22 => {
            let length = reader.short();
            let mut values = vec![];
            for _ in 0..length {
                values.push(read_object(reader));
            }
            Object::NotImplemented
        }
        23 => {
            reader.byte();
            Object::NotImplemented
        }
        other => {
            eprintln!("Unknown object: {other}");
            Object::Unknown
        }
    }
}

// TODO
pub fn write_object(buf: &mut Vec<u8>, object: Object) {
    match object {
        Object::Null => {
            write_byte(buf, 0u8);
        }
        Object::Int(value) => {
            write_byte(buf, 1u8);
            write_int(buf, value);
        }
        Object::Long(value) => {
            write_byte(buf, 2u8);
            write_long(buf, value)
        }
        Object::Float(value) => {
            write_byte(buf, 3u8);
            write_float(buf, value);
        }
        Object::String(value) => {
            write_byte(buf, 4u8);
            write_string(buf, &*value.unwrap());
        }
        // Content
        Object::IntSequence(values) => {
            write_byte(buf, 6u8);
            write_short(buf, values.len() as i16);
            for value in values {
                write_int(buf, value);
            }
        }
        Object::Point2(value) => {
            write_byte(buf, 7u8);
            write_int(buf, value.x as u32);
            write_int(buf, value.y as u32);
        }
        Object::Point2Array(values) => {
            write_byte(buf, 8u8);
            write_byte(buf, values.len() as u8);
            for value in values {
                write_int(buf, value.pack() as u32);
            }
        }
        // TechNode
        Object::Boolean(value) => {
            write_byte(buf, 10u8);
            write_bool(buf, value);
        }
        Object::Double(value) => {
            write_byte(buf, 11u8);
            write_double(buf, value);
        }
        // BuildingBox
        // LAccess
        Object::ByteArray(values) => {
            write_byte(buf, 14u8);
            write_int(buf, values.len() as u32);
            for value in values {
                write_byte(buf, value);
            }
        }
        Object::BooleanArray(values) => {
            write_byte(buf, 16u8);
            write_int(buf, values.len() as u32);
            for value in values {
                write_bool(buf, value);
            }
        }
        // Unit
        Object::Vec2Array(values) => {
            write_byte(buf, 18u8);
            write_short(buf, values.len() as i16);
            for value in values {
                write_float(buf, value.x);
                write_float(buf, value.y);
            }
        }
        Object::Vec2(value) => {
            write_byte(buf, 19u8);
            write_float(buf, value.x);
            write_float(buf, value.y);
        }
        // Team
        // int[]
        // Object[]
        // UnitCommand
        Object::NotImplemented => {}
        Object::Unknown => {}
    }
}

pub fn read_string_map(reader: &mut Reader) -> HashMap<String, Option<String>> {
    let mut data = HashMap::new();

    let size = reader.short();
    for _ in 0..size {
        let key = read_string(reader).unwrap();
        let value = read_string(reader);
        data.insert(key, value);
    }

    data
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum KickReason {
    Kick = 0,
    ClientOutdated = 1,
    ServerOutdated = 2,
    Banned = 3,
    GameOver = 4,
    RecentKick = 5,
    NameInUse = 6,
    IdInUse = 7,
    NameEmpty = 8,
    CustomClient = 9,
    ServerClose = 10,
    Vote = 11,
    TypeMismatch = 12,
    Whitelist = 13,
    PlayerLimit = 14,
    ServerRestarting = 15,
}

pub fn read_kick(reader: &mut Reader) -> Option<KickReason> {
    KickReason::try_from(reader.byte()).ok()
}

pub fn write_kick(buf: &mut Vec<u8>, reason: KickReason) {
    buf.push(reason as u8);
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub x: i16,
    pub y: i16,
}

pub fn read_tile(reader: &mut Reader) -> Tile {
    let x = reader.short();
    let y = reader.short();
    Tile { x, y }
}

pub fn write_tile(buf: &mut Vec<u8>, tile: Tile) {
    write_short(buf, tile.x);
    write_short(buf, tile.y);
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub unit_type: u8,
    pub id: u32,
}

pub fn read_unit(reader: &mut Reader) -> Unit {
    let unit_type = reader.byte();
    let id = reader.int();
    Unit { unit_type, id }
}

pub fn write_unit(buf: &mut Vec<u8>, unit: Unit) {
    write_int(buf, unit.id);
    write_byte(buf, unit.unit_type);
}

#[derive(Debug, Clone)]
pub struct Items {
    pub id: i16,
    pub count: u32,
}

pub fn read_items(reader: &mut Reader) -> Items {
    let id = reader.short();
    let count = reader.int();
    Items { id, count }
}

#[derive(Debug, Clone)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

pub fn read_vec2(reader: &mut Reader) -> Vec2 {
    let x = reader.float();
    let y = reader.float();
    Vec2 { x, y }
}

#[derive(Debug, Clone)]
pub struct Vec2Nullable {
    pub x: f32,
    pub y: f32,
}

pub fn read_vec2_nullable(reader: &mut Reader) -> Vec2 {
    // TODO  (isNaN(x) || isNaN(y)) ? null : {x, y}
    // How does NaN even work
    let x = reader.float();
    let y = reader.float();
    Vec2 { x, y }
}

pub fn read_command(reader: &mut Reader) -> Option<u8> {
    let value = reader.byte();
    if value == 255 { None } else { Some(value) }
}
