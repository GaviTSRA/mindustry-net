use num_enum::{IntoPrimitive, TryFromPrimitive};

pub fn read_byte(buf: &mut Vec<u8>) -> u8 {
  buf.drain(..1).collect::<Vec<u8>>()[0]
}

pub fn write_byte(buf: &mut Vec<u8>, value: u8) {
  buf.push(value);
}

pub fn read_bytes(buf: &mut Vec<u8>) -> Vec<u8> {
  let length = read_short(buf);
  buf.drain(..length as usize).collect::<Vec<u8>>()
}

pub fn read_short(buf: &mut Vec<u8>) -> u16 {
  let bytes = buf.drain(..2).collect::<Vec<_>>();
  u16::from_be_bytes([bytes[0], bytes[1]])
}

pub fn write_short(buf: &mut Vec<u8>, short: u16) {
  buf.extend_from_slice(&short.to_be_bytes());
}

pub fn read_int(buf: &mut Vec<u8>) -> u32 {
  let bytes = buf.drain(..4).collect::<Vec<_>>();
  u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub fn write_int(buf: &mut Vec<u8>, int: u32) {
  buf.extend_from_slice(&int.to_be_bytes());
}

pub fn read_long(buf: &mut Vec<u8>) -> u64 {
  let bytes = buf.drain(..8).collect::<Vec<_>>();
  u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],  bytes[6], bytes[7]])
}

pub fn write_long(buf: &mut Vec<u8>, long: u64) {
  buf.extend_from_slice(&long.to_be_bytes());
}

pub fn read_float(buf: &mut Vec<u8>) -> f32 {
  let bytes = buf.drain(..4).collect::<Vec<_>>();
  f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub fn write_float(buf: &mut Vec<u8>, data: f32) {
  buf.extend_from_slice(&data.to_be_bytes());
}

pub fn read_double(buf: &mut Vec<u8>) -> f64 {
  let bytes = buf.drain(..8).collect::<Vec<_>>();
  f64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
}

pub fn read_prefixed_string(buf: &mut Vec<u8>) -> Option<String> {
  if buf.is_empty() {
    return None;
  }

  let has_string = read_byte(buf) != 0;
  if !has_string {
    return None;
  }

  if buf.len() < 2 {
    return None;
  }
  let length = read_short(buf) as usize;

  if buf.len() < length {
    return None;
  }
  let string_bytes: Vec<u8> = buf.drain(..length).collect();
  String::from_utf8(string_bytes).ok()
}

pub fn read_string(buf: &mut Vec<u8>) -> Option<String> {
  let length = read_short(buf);
  if length == 0 {
    return None;
  }
  let string_bytes: Vec<u8> = buf.drain(..length as usize).collect();
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
  String(String),

  Unknown,

  NotImplemented
}

// TODO
pub fn read_object(buf: &mut Vec<u8>) -> Object {
  let object_type = read_byte(buf);

  match object_type {
    0 => Object::Null,
    1 => Object::Int(read_int(buf)),
    2 => Object::Long(read_long(buf)),
    3 => Object::Float(read_float(buf)),
    4 => Object::String(read_prefixed_string(buf).unwrap()),
    5 => {
      read_byte(buf);
      read_short(buf);
      Object::NotImplemented
    }
    6 => {
      let length = read_short(buf);
      let mut values = vec![];
      for _ in 0..length {
        values.push(read_int(buf));
      }
      Object::NotImplemented
    }
    7 => {
      read_int(buf);
      read_int(buf);
      Object::NotImplemented
    }
    8 => {
      let length = read_short(buf);
      let mut values = vec![];
      for _ in 0..length {
        values.push(read_int(buf));
      }
      Object::NotImplemented
    }
    9 => {
      read_byte(buf);
      read_short(buf);
      Object::NotImplemented
    }
    10 => {
      read_byte(buf);
      Object::NotImplemented
    }
    11 => {
      read_double(buf);
      Object::NotImplemented
    }
    12 => {
      read_int(buf);
      Object::NotImplemented
    }
    13 => {
      read_short(buf);
      Object::NotImplemented
    }
    14 => {
      let length = read_short(buf);
      let mut values = vec![];
      for _ in 0..length {
        values.push(read_byte(buf));
      }
      Object::NotImplemented
    }
    15 => {
      read_byte(buf);
      Object::NotImplemented
    }
    16 => {
      let length = read_short(buf);
      let mut values = vec![];
      for _ in 0..length {
        values.push(read_byte(buf));
      }
      Object::NotImplemented
    }
    17 => {
      read_int(buf);
      Object::NotImplemented
    }
    18 => {
      let length = read_short(buf);
      for _ in 0..length {
        read_float(buf);
        read_float(buf);
      }
      Object::NotImplemented
    }
    19 => {
      read_float(buf);
      read_float(buf);
      Object::NotImplemented
    }
    20 => {
      read_byte(buf);
      Object::NotImplemented
    }
    21 => {
      let length = read_short(buf);
      let mut values = vec![];
      for _ in 0..length {
        values.push(read_int(buf));
      }
      Object::NotImplemented
    }
    22 =>  {
      let length = read_short(buf);
      let mut values = vec![];
      for _ in 0..length {
        values.push(read_object(buf));
      }
      Object::NotImplemented
    }
    23 => {
      read_byte(buf);
      Object::NotImplemented
    }
    _ => Object::Unknown,
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
      write_string(buf, &value);
    }
    Object::NotImplemented => {}
    Object::Unknown => {}
  }
}

pub fn read_string_map(mut buf: &mut Vec<u8>) {
  let size = read_short(&mut buf);
  for _ in 0..size {
    let key = read_string(&mut buf);
    let value = read_string(&mut buf);
  }
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
  ServerRestarting = 15
}

pub fn read_kick(buf: &mut Vec<u8>) -> Option<KickReason> {
  KickReason::try_from(buf[0]).ok()
}

pub fn write_kick(buf: &mut Vec<u8>, reason: KickReason) {
  buf.push(reason as u8);
}

#[derive(Debug, Clone)]
pub struct Tile {
  pub x: u16,
  pub y: u16
}

pub fn read_tile(buf: &mut Vec<u8>) -> Tile {
  let x = read_short(buf);
  let y = read_short(buf);
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

pub fn read_unit(buf: &mut Vec<u8>) -> Unit {
  let unit_type = read_byte(buf);
  let id = read_int(buf);
  Unit {
    unit_type,
    id,
  }
}

pub fn write_unit(buf: &mut Vec<u8>, unit: Unit) {
  write_int(buf, unit.id);
  write_byte(buf, unit.unit_type);
}

#[derive(Debug)]
pub struct Items {
  id: u16,
  count: u32
}

pub fn read_items(buf: &mut Vec<u8>) -> Items {
  let id = read_short(buf);
  let count =  read_int(buf);
  Items { id, count }
}

#[derive(Debug)]
pub struct Vec2 {
  x: f32,
  y: f32
}

pub fn read_vec2(buf: &mut Vec<u8>) -> Vec2 {
  let x = read_float(buf);
  let y = read_float(buf);
  Vec2 { x, y }
}
