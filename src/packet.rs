use std::collections::HashMap;
use std::io::Read;
use tokio::io::AsyncReadExt;
use base64::engine::general_purpose;
use std::sync::Arc;
use tokio::net::UdpSocket;
use base64::Engine;
use flate2::read::ZlibDecoder;
use lz4::block::{compress, decompress};
use tokio::net::tcp::OwnedReadHalf;
use crate::type_io::{read_double, read_float, read_int, read_kick, read_long, read_short, read_prefixed_string, read_string_map, write_float, write_int, write_short, write_string, KickReason, read_string, read_byte, read_bytes, Unit, write_byte, write_unit};
use crate::unit_io::{read_full_unit, write_plans, FullUnit, Plan};

#[derive(Debug)]
pub enum PacketError {
  FailedToReadLength,
  FailedToReadData,
  UnknownFrameworkPacket,
  DecompressionFailed,
  WorldDataDecompressionFailed
}

pub enum AnyPacket {
  Framework(FrameworkPacket),
  Regular(Packet)
}

#[derive(Debug)]
pub enum FrameworkPacket {
  DiscoverHost,
  KeepAlive,
  RegisterUDP(u32),
  RegisterTCP(u32)
}

#[derive(Debug)]
pub enum Packet {
  // [00] Stream begin
  StreamBegin { id: u32, total: u32, stream_type: u8 },
  // [01] Stream Chunk
  StreamChunk { id: u32, data: Vec<u8> },
  // [02] Completed world stream
  WorldStream { // TODO
    /* rules */ /* map */ wave: u32, wave_time: f32, tick: f64, seed0: u64, seed1: u64, id: u32 /* more */
  },
  // [03] Connect to server
  Connect {
    version: u32, client: String, name: String, lang: String, usid: String, uuid: String,
    mobile: bool, color: Vec<u8>, mods: Vec<String>
  },
  // [10] Begin block place
  BeginPlaceCall {
    unit: Unit,
    result: u16,
    team: u8,
    x: u32,
    y: u32,
    rotation: u32
  },
  // [18] Client Snapshot
  ClientSnapshot { 
    snapshot_id: u32, unit_id: u32, dead: bool, x: f32, y: f32, pointer_x: f32, pointer_y: f32, 
    rotation: f32, base_rotation: f32, x_velocity: f32, y_velocity: f32, mining_x: u16, mining_y: u16,
    boosting: bool, shooting: bool, chatting: bool, building: bool, plans: Vec<Plan>,
    view_x: f32, view_y: f32, view_width: f32, view_height: f32,
  },
  // [22] Confirm connect call
  ConnectCallConfirm,
  // [34] Entity snapshot call
  EntitySnapshot { units: HashMap<u32, FullUnit> },
  // [44] Kick with a custom message
  KickCall { reason: String },
  // [45] Kick with a preset message
  KickCall2 { reason: KickReason },
  // [59] Spawn call
  SpawnCall { tile_x: u16, tile_y: u16, entity: u32 },
  // [71] Send a chat message to server
  SendChatMessageCall { message: String },
  // [73] Received a chat message from server
  SendMessageCall2 { message: String, unformatted: Option<String>, sender: u32 },
  // [86] Set position call
  // SetPositionCall { x: f32, y: f32 },
  Other(u8),
}

pub async fn read_packet_tcp(
  stream: &mut OwnedReadHalf
) -> Result<AnyPacket, PacketError> {
  let mut buf = [0u8; 2];
  let length = match stream.read_exact(&mut buf).await {
    Ok(_) => {
      u16::from_be_bytes(buf)
    }
    Err(e) => {
      eprintln!("{}", e);
      return Err(PacketError::FailedToReadLength)
    }
  };

  let mut buf =  vec![0u8; length as usize];
  match stream.read_exact(&mut buf).await {
    Ok(_) => {}
    Err(e) => {
      eprintln!("{}", e);
      return Err(PacketError::FailedToReadData)
    }
  }

  parse_packet(buf)
}

pub async fn read_packet_udp(
  socket: &mut Arc<UdpSocket>
) -> Result<AnyPacket, PacketError> {
  let mut buf = [0u8; 32768];
  let length = socket.recv(&mut buf).await.unwrap();
  let data = &buf[..length];
  parse_packet(Vec::from(data))
}

pub fn parse_packet(mut buf: Vec<u8>) -> Result<AnyPacket, PacketError> {
  let id = read_byte(&mut buf);

  if id == 254 {
    Ok(AnyPacket::Framework(parse_framework_packet(buf)?))
  } else {
    let data_length = read_short(&mut buf);

    let compressed = read_byte(&mut buf);

    if compressed == 1 {
      buf = match decompress(&buf, Some(data_length as i32)) {
        Ok(buf) => buf,
        Err(e) => {
          eprintln!("{e}");
          return Err(PacketError::DecompressionFailed)
        }
      };
    }
    Ok(AnyPacket::Regular(parse_regular_packet(id, buf)?))
  }
}

fn parse_framework_packet(mut buf: Vec<u8>) -> Result<FrameworkPacket, PacketError> {
  let id = read_byte(&mut buf);
  Ok(match id {
    1 => FrameworkPacket::DiscoverHost,
    2 => FrameworkPacket::KeepAlive,
    3 => {
      let bytes =  buf.drain(..4).collect::<Vec<u8>>();
      let mut data_buf = [0u8; 4];
      data_buf.copy_from_slice(&bytes);
      let data = u32::from_be_bytes(data_buf);
      FrameworkPacket::RegisterUDP(data)
    },
    4 => {
      let bytes =  buf.drain(..4).collect::<Vec<u8>>();
      let mut data_buf = [0u8; 4];
      data_buf.copy_from_slice(&bytes);
      let data = u32::from_be_bytes(data_buf);
      FrameworkPacket::RegisterTCP(data)
    }
    _ => return Err(PacketError::UnknownFrameworkPacket)
  })
}

pub fn parse_regular_packet(id: u8, mut buf: Vec<u8>) -> Result<Packet, PacketError> {
  //println!("{id}");
  
  match id {
    0 => {
      let id = read_int(&mut buf);
      let total = read_int(&mut buf);
      let stream_type = read_byte(&mut buf);
      Ok(Packet::StreamBegin { id, total, stream_type })
    }
    1 => {
      let id = read_int(&mut buf);
      let length = read_short(&mut buf);
      let data = buf.drain(..(length as usize)).collect::<Vec<u8>>();
      Ok(Packet::StreamChunk { id, data })
    }
    2 => {
      let mut decoder = ZlibDecoder::new(&*buf);
      let mut data = Vec::new();
      match decoder.read_to_end(&mut data) {
        Ok(_) =>  {},
        Err(e) => {
          eprintln!("{e}");
          return Err(PacketError::WorldDataDecompressionFailed)
        }
      }

      let rules_json = read_string(&mut data); // TODO
      let map = read_string_map(&mut data); // TODO
      let wave = read_int(&mut data);
      let wave_time = read_float(&mut data);
      let tick = read_double(&mut data);
      let seed0 = read_long(&mut data);
      let seed1 = read_long(&mut data);
      let id = read_int(&mut data);
      
      // TODO
      read_short(&mut data);
      read_byte(&mut data);
      read_byte(&mut data);
      read_int(&mut data);
      read_byte(&mut data);
      read_float(&mut data);
      read_float(&mut data);
      read_prefixed_string(&mut data);
      read_byte(&mut data);
      read_byte(&mut data);
      read_byte(&mut data);
      read_byte(&mut data);
      read_int(&mut data);
      read_float(&mut data);
      read_float(&mut data);

      println!("Remaining data: {}", data.len()); // TODO
      
      Ok(Packet::WorldStream {
        wave,
        wave_time,
        tick,
        seed0,
        seed1,
        id,
      })
    }
    34 => {
      let mut units = HashMap::new();

      let amount = read_short(&mut buf);
      let mut data = read_bytes(&mut buf);

      for _ in 0..amount {
        let id = read_int(&mut data);
        let unit_type = read_byte(&mut data);
        let unit = read_full_unit(&mut data, unit_type, false);
        units.insert(id, unit);
      }

      Ok(Packet::EntitySnapshot { units })
    }
    44 => {
      let reason = read_prefixed_string(&mut buf).unwrap();
      Ok(Packet::KickCall { reason })
    }
    45 => {
      let reason = read_kick(&mut buf).unwrap();
      Ok(Packet::KickCall2 { reason })
    }
    59 => {
      let tile_x = read_short(&mut buf);
      let tile_y = read_short(&mut buf);
      let entity = read_int(&mut buf);
      Ok(Packet::SpawnCall { tile_x, tile_y, entity })
    }
    73 => {
      let message = read_prefixed_string(&mut buf).unwrap();
      let unformatted = read_prefixed_string(&mut buf);
      let sender = read_int(&mut buf);
      Ok(Packet::SendMessageCall2 {
        message, unformatted, sender
      }) 
    }
    id => Ok(Packet::Other(id))
  }
}


pub fn write_framework_packet(packet: FrameworkPacket) -> Vec<u8> {
  let mut data: Vec<u8> = vec![];

  match packet {
    FrameworkPacket::DiscoverHost => {}, // TODO
    FrameworkPacket::KeepAlive => {
      data.extend_from_slice(&vec![0x00, 0x06, 0xFE, 0x03, 0x00, 0x00, 0x00, 0x00]);
    },
    FrameworkPacket::RegisterUDP(id) => {
      data.push(0xFE);
      data.push(0x03);
      data.extend_from_slice(&id.to_be_bytes());
    }
    FrameworkPacket::RegisterTCP(id) => {
      data.push(0xFE);
      data.push(0x04);
      data.extend_from_slice(&id.to_be_bytes());
    }
  };
  data
}

pub fn write_packet(packet: Packet) -> Vec<u8> {
  let mut data: Vec<u8> = vec![];

  let id = match packet {
    Packet::Connect { version, client, name, lang, usid, uuid, mobile, color, mods} => {
      write_int(&mut data, version);
      write_string(&mut data, &client);
      write_string(&mut data, &name);
      write_string(&mut data, &lang);
      write_string(&mut data, &usid);

      let uuid_bytes = general_purpose::STANDARD
          .decode(uuid)
          .expect("Invalid base64 UUID");
      data.extend_from_slice(&uuid_bytes);

      // Should be equivalent to this, for some reason the js library just puts 0's
      // buf.putLong(crc32(uuid_buf));
      data.extend_from_slice(&[0x00; 8]);

      data.push(mobile as u8);

      data.extend_from_slice(&color);

      data.push(mods.len() as u8);
      for entry in mods {
        write_string(&mut data, &entry);
      }

      3
    }
    Packet::BeginPlaceCall { unit, result, team, x, y, rotation } => {
      write_unit(&mut data, unit);
      write_short(&mut data, result);
      write_byte(&mut data, team);
      write_int(&mut data, x);
      write_int(&mut data, y);
      write_int(&mut data, rotation);

      10
    }
    Packet::ClientSnapshot { 
      snapshot_id, unit_id, dead, x, y, pointer_x, pointer_y, 
      rotation, base_rotation, x_velocity, y_velocity, mining_x, mining_y, 
      boosting, shooting, chatting, building, view_x, view_y,  plans,
      view_width, view_height, 
    } => {
      write_int(&mut data, snapshot_id);
      write_int(&mut data, unit_id);
      write_byte(&mut data, dead as u8);
      write_float(&mut data, x);
      write_float(&mut data, y);
      write_float(&mut data, pointer_x);
      write_float(&mut data, pointer_y);
      write_float(&mut data, rotation);
      write_float(&mut data, base_rotation);
      write_float(&mut data, x_velocity);
      write_float(&mut data, y_velocity);
      write_short(&mut data, mining_x);
      write_short(&mut data, mining_y);
      write_byte(&mut data, boosting as u8);
      write_byte(&mut data, shooting as u8);
      write_byte(&mut data, chatting as u8);
      write_byte(&mut data, building as u8);
      write_plans(&mut data, plans);
      write_float(&mut data, view_x);
      write_float(&mut data, view_y);
      write_float(&mut data, view_width);
      write_float(&mut data, view_height);
      18
    }
    Packet::ConnectCallConfirm => {
      22
    }
    Packet::SendChatMessageCall { message } => {
      write_string(&mut data, &message);
      71
    }
    _ => 0
  };

  let mut buf: Vec<u8> = vec![];
  let length = data.len() as u16 + 4;

  if length > 35 {
    let uncompressed_length = data.len() as u16;
    data = compress(&data, None, false).unwrap();
    let length = data.len() as u16 + 4;

    write_short(&mut buf, length);
    buf.push(id);

    write_short(&mut buf, uncompressed_length);
    buf.push(0x01);
  } else {
    write_short(&mut buf, length);
    buf.push(id);
    write_short(&mut buf, data.len() as u16);
    buf.push(0x00);
  }

  buf.extend_from_slice(&data);
  buf
}