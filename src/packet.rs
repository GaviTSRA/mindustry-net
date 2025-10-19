use crate::save_io::{Map, read_content_header, read_map, read_markers, read_team_blocks};
use crate::type_io::{
    KickReason, Object, Reader, Tile, Unit, read_kick, read_object, read_prefixed_string,
    read_string, read_string_map, read_tile, read_unit, write_byte, write_float, write_int,
    write_string, write_unsigned_short,
};
use crate::unit_io::{FullUnit, Plan, read_full_unit, write_plans};
use base64::Engine;
use base64::engine::general_purpose;
use flate2::read::ZlibDecoder;
use lz4::block::{compress, decompress};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::UdpSocket;
use tokio::net::tcp::OwnedReadHalf;

#[derive(Debug)]
pub enum PacketError {
    FailedToReadLength,
    FailedToReadData,
    UnknownFrameworkPacket,
    DecompressionFailed,
    WorldDataDecompressionFailed,
}

pub enum AnyPacket {
    Framework(FrameworkPacket),
    Regular(Packet),
}

#[derive(Debug)]
pub enum FrameworkPacket {
    DiscoverHost,
    KeepAlive,
    RegisterUDP(u32),
    RegisterTCP(u32),
}

#[derive(Debug)]
pub enum Packet {
    // [000] Stream begin
    StreamBegin {
        id: u32,
        total: u32,
        stream_type: u8,
    },
    // [001] Stream Chunk
    StreamChunk {
        id: u32,
        data: Vec<u8>,
    },
    // [002] Completed world stream
    WorldStream {
        // TODO
        /* rules */ /* map */ wave: u32,
        wave_time: f32,
        tick: f64,
        seed0: u64,
        seed1: u64,
        id: u32,
        content_map: HashMap<String, Vec<String>>, /* more */
        map: Map,
    },
    // [003] Connect to server
    Connect {
        version: u32,
        client: String,
        name: String,
        lang: String,
        usid: String,
        uuid: String,
        mobile: bool,
        color: Vec<u8>,
        mods: Vec<String>,
    },
    // [004] Admin Request
    // [005] Announce
    // [006] Assembler Drone Spawned
    // [007] Assembler Unit Spawned
    // [008] Auto Door Toggle
    // [009] Begin Break
    BeginBreak {
        unit: Unit,
        team: u8,
        x: u32,
        y: u32,
    },
    // [010] Begin Place
    BeginPlace {
        unit: Unit,
        result: u16,
        team: u8,
        x: u32,
        y: u32,
        rotation: u32,
    },
    // [011] Block Snapshot
    BlockSnapshot {
        amount: i16,
        data: Vec<u8>,
    },
    // [012] Build Destroyed
    // [013] Build Health Update
    // [014] Building Control Select
    // [015] Clear Items
    // [016] Client Packet Reliable
    // [017] Client Packet Unreliable
    // [018] Client Snapshot
    ClientSnapshot {
        snapshot_id: u32,
        unit_id: u32,
        dead: bool,
        x: f32,
        y: f32,
        pointer_x: f32,
        pointer_y: f32,
        rotation: f32,
        base_rotation: f32,
        x_velocity: f32,
        y_velocity: f32,
        mining_x: u16,
        mining_y: u16,
        boosting: bool,
        shooting: bool,
        chatting: bool,
        building: bool,
        plans: Vec<Plan>,
        view_x: f32,
        view_y: f32,
        view_width: f32,
        view_height: f32,
    },
    // [019] Command Building
    // [020] Command Units
    // [021] Connect Call
    // [022] Connect Call Confirm
    ConnectCallConfirm,
    // [023] Construct Finish
    ContructFinishCall {
        tile: Tile,
        block: i16,
        builder: Unit,
        rotation: u8,
        team: u8,
        config: Object,
    },
    // [024]
    // [025]
    // [026]
    // [027]
    // [028]
    // [029]
    // [030]
    // [031]
    // [032]
    // [033]
    // [034] Entity Snapshot
    EntitySnapshot {
        units: HashMap<u32, FullUnit>,
    },
    // [035]
    // [036]
    // [037]
    // [038]
    // [039]
    // [040]
    // [041]
    // [042]
    // [043]
    // [044] Kick with a custom message
    KickCall {
        reason: String,
    },
    // [045] Kick with a preset message
    KickCall2 {
        reason: KickReason,
    },
    // [046]
    // [047]
    // [048]
    // [049]
    // [050]
    // [051]
    // [052]
    // [053]
    // [054]
    // [055]
    // [056]
    // [057]
    // [058]
    // [059] Spawn call
    SpawnCall {
        tile_x: i16,
        tile_y: i16,
        entity: u32,
    },
    // [060]
    // [061]
    // [062]
    // [063]
    // [064]
    // [065]
    // [066]
    // [067]
    // [068]
    // [069] Rotate Block Call
    RotateBlockCall {
        entity: u32,
        tile: Tile,
        rotation: u8,
    },
    // [070]
    // [071] Send a chat message to server
    SendChatMessageCall {
        message: String,
    },
    // [072]
    // [073] Received a chat message from server
    SendMessageCall2 {
        message: String,
        unformatted: Option<String>,
        sender: u32,
    },
    // [074]
    // [075]
    // [076]
    // [077]
    // [078]
    // [079]
    // [080]
    // [081]
    // [082]
    // [083]
    // [084]
    // [085]
    // [086] Set position call
    // SetPositionCall { x: f32, y: f32 },
    // [088]
    // [089]
    // [090]
    // [091]
    // [092]
    // [093]
    // [094] StateSnapshot
    StateSnapshot {
        wave_time: f32,
        wave: u32,
        enemies: u32,
        paused: bool,
        game_over: bool,
        time_data: u32,
        tps: u8,
        rand0: u64,
        rand1: u64,
        core_data: Vec<u8>,
    },
    // [095]
    // [096]
    // [097]
    // [098]
    // [099] TileConfigCall
    TileConfigCall {
        player: u32,
        tile: Tile,
        value: Object,
    },
    // [100]
    // [101]
    // [102]
    // [103]
    // [104]
    // [105]
    // [106]
    // [107]
    // [108]
    // [109]
    // [110]
    // [111]
    // [112]
    // [113]
    // [114]
    // [115]
    // [116]
    // [117]
    // [118]
    Other(u8),
}

pub async fn read_packet_tcp(
    stream: &mut OwnedReadHalf,
    content_map: &Option<HashMap<String, Vec<String>>>,
) -> Result<AnyPacket, PacketError> {
    let mut buf = [0u8; 2];
    let length = match stream.read_exact(&mut buf).await {
        Ok(_) => u16::from_be_bytes(buf),
        Err(e) => {
            eprintln!("{}", e);
            return Err(PacketError::FailedToReadLength);
        }
    };

    let mut buf = vec![0u8; length as usize];
    match stream.read_exact(&mut buf).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            return Err(PacketError::FailedToReadData);
        }
    }

    parse_packet(buf, content_map)
}

pub async fn read_packet_udp(
    socket: &mut Arc<UdpSocket>,
    content_map: &Option<HashMap<String, Vec<String>>>,
) -> Result<AnyPacket, PacketError> {
    let mut buf = [0u8; 32768];
    let length = socket.recv(&mut buf).await.unwrap();
    let data = &buf[..length];

    parse_packet(Vec::from(data), content_map)
}

pub fn parse_packet(
    mut buf: Vec<u8>,
    content_map: &Option<HashMap<String, Vec<String>>>,
) -> Result<AnyPacket, PacketError> {
    let mut reader = Reader::new(buf);

    let id = reader.byte();

    if id == 254 {
        Ok(AnyPacket::Framework(parse_framework_packet(reader)?))
    } else {
        let data_length = reader.short();

        let compressed = reader.byte();

        if compressed == 1 {
            buf = match decompress(&*reader.read_remaining(), Some(data_length as i32)) {
                Ok(buf) => buf,
                Err(e) => {
                    eprintln!("{e}");
                    return Err(PacketError::DecompressionFailed);
                }
            };
            reader = Reader::new(buf);
        }
        Ok(AnyPacket::Regular(parse_regular_packet(
            id,
            reader,
            content_map,
        )?))
    }
}

fn parse_framework_packet(mut reader: Reader) -> Result<FrameworkPacket, PacketError> {
    let id = reader.byte();
    Ok(match id {
        1 => FrameworkPacket::DiscoverHost,
        2 => FrameworkPacket::KeepAlive,
        3 => {
            let bytes = reader.bytes(4);
            let mut data_buf = [0u8; 4];
            data_buf.copy_from_slice(&bytes);
            let data = u32::from_be_bytes(data_buf);
            FrameworkPacket::RegisterUDP(data)
        }
        4 => {
            let bytes = reader.bytes(4);
            let mut data_buf = [0u8; 4];
            data_buf.copy_from_slice(&bytes);
            let data = u32::from_be_bytes(data_buf);
            FrameworkPacket::RegisterTCP(data)
        }
        _ => return Err(PacketError::UnknownFrameworkPacket),
    })
}

pub fn parse_regular_packet(
    id: u8,
    mut reader: Reader,
    content_map: &Option<HashMap<String, Vec<String>>>,
) -> Result<Packet, PacketError> {
    // println!("{id}");

    let result = match id {
        0 => {
            let id = reader.int();
            let total = reader.int();
            let stream_type = reader.byte();
            Ok(Packet::StreamBegin {
                id,
                total,
                stream_type,
            })
        }
        1 => {
            let id = reader.int();
            let length = reader.short();
            let data = reader.bytes(length as usize);
            Ok(Packet::StreamChunk { id, data })
        }
        2 => {
            let remaining = reader.read_remaining();
            let mut decoder = ZlibDecoder::new(&*remaining);
            let mut data = Vec::new();
            match decoder.read_to_end(&mut data) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{e}");
                    return Err(PacketError::WorldDataDecompressionFailed);
                }
            }
            reader = Reader::new(data);

            let rules_json = read_string(&mut reader); // TODO
            let map = read_string_map(&mut reader); // TODO
            let wave = reader.int();
            let wave_time = reader.float();
            let tick = reader.double();
            let seed0 = reader.long();
            let seed1 = reader.long();
            let id = reader.int();

            reader.short();
            reader.byte();
            reader.byte();
            reader.int();
            reader.byte();
            reader.float();
            reader.float();
            read_prefixed_string(&mut reader);
            reader.byte();
            reader.byte();
            reader.byte();
            reader.byte();
            reader.int();
            reader.float();
            reader.float();

            let content_map = read_content_header(&mut reader);

            let default_content_map_path = PathBuf::from("content-map.json");
            let default_content_map_data = serde_json::to_string(&content_map).unwrap();
            fs::write(&default_content_map_path, default_content_map_data).unwrap();

            let map = read_map(&mut reader, &content_map);
            let team_blocks = read_team_blocks(&mut reader);
            println!("{team_blocks:?}");

            println!("{:?}", reader.read_remaining());

            // let markers = read_markers(&mut reader);
            // println!("{markers:?}");
            // let custom_chunks = read_custom_chunks(&mut reader);
            // println!("{custom_chunks:?}");

            println!("Remaining data: {}", reader.remaining()); // TODO

            Ok(Packet::WorldStream {
                wave,
                wave_time,
                tick,
                seed0,
                seed1,
                id,
                content_map,
                map,
            })
        }
        9 => Ok(Packet::BeginBreak {
            unit: read_unit(&mut reader),
            team: reader.byte(),
            x: reader.int(),
            y: reader.int(),
        }),
        10 => Ok(Packet::BeginPlace {
            unit: read_unit(&mut reader),
            result: reader.unsigned_short(),
            team: reader.byte(),
            x: reader.int(),
            y: reader.int(),
            rotation: reader.int(),
        }),
        11 => {
            let amount = reader.short();
            let data_length = reader.short();
            let data = reader.bytes(data_length as usize);
            println!("Did not read {}", reader.remaining());
            Ok(Packet::BlockSnapshot { amount, data })
        }
        23 => {
            let tile = read_tile(&mut reader);
            let block = reader.short();
            let builder = read_unit(&mut reader);
            let rotation = reader.byte();
            let team = reader.byte();
            let config = read_object(&mut reader);
            Ok(Packet::ContructFinishCall {
                tile,
                block,
                builder,
                rotation,
                team,
                config,
            })
        }
        34 => {
            let mut units = HashMap::new();

            let amount = reader.short();
            let byte_count = reader.short();
            let data = reader.bytes(byte_count as usize);

            let mut unit_reader = Reader::new(data);

            for _ in 0..amount {
                let id = unit_reader.int();
                let unit_type = unit_reader.byte();
                let unit = read_full_unit(&mut unit_reader, unit_type, false, content_map);
                units.insert(id, unit);
            }

            Ok(Packet::EntitySnapshot { units })
        }
        44 => {
            let reason = read_prefixed_string(&mut reader).unwrap();
            Ok(Packet::KickCall { reason })
        }
        45 => {
            let reason = read_kick(&mut reader).unwrap();
            Ok(Packet::KickCall2 { reason })
        }
        59 => {
            let tile_x = reader.short();
            let tile_y = reader.short();
            let entity = reader.int();
            Ok(Packet::SpawnCall {
                tile_x,
                tile_y,
                entity,
            })
        }
        69 => {
            let entity = reader.int();
            let tile = read_tile(&mut reader);
            let rotation = reader.byte();
            Ok(Packet::RotateBlockCall {
                entity,
                tile,
                rotation,
            })
        }
        73 => {
            let message = read_prefixed_string(&mut reader).unwrap();
            let unformatted = read_prefixed_string(&mut reader);
            let sender = reader.int();
            Ok(Packet::SendMessageCall2 {
                message,
                unformatted,
                sender,
            })
        }
        94 => {
            let wave_time = reader.float();
            let wave = reader.int();
            let enemies = reader.int();
            let paused = reader.bool();
            let game_over = reader.bool();
            let time_data = reader.int();
            let tps = reader.byte();
            let rand0 = reader.long();
            let rand1 = reader.long();

            let length = reader.short();
            let core_data = reader.bytes(length as usize);

            Ok(Packet::StateSnapshot {
                wave_time,
                wave,
                enemies,
                paused,
                game_over,
                time_data,
                tps,
                rand0,
                rand1,
                core_data,
            })
        }
        99 => {
            let player = reader.int();
            let tile = read_tile(&mut reader);
            let value = read_object(&mut reader);
            println!("Config: {tile:?} {value:?}");
            Ok(Packet::TileConfigCall {
                player,
                tile,
                value,
            })
        }
        id => Ok(Packet::Other(id)),
    };

    if reader.remaining() != 0 {
        eprintln!(
            "Did not read complete packet: ID {id}, {} bytes remain",
            reader.remaining()
        );
    }

    result
}

pub fn write_framework_packet(packet: FrameworkPacket) -> Vec<u8> {
    let mut data: Vec<u8> = vec![];

    match packet {
        FrameworkPacket::DiscoverHost => {} // TODO
        FrameworkPacket::KeepAlive => {
            data.extend_from_slice(&vec![0x00, 0x06, 0xFE, 0x03, 0x00, 0x00, 0x00, 0x00]);
        }
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
        Packet::Connect {
            version,
            client,
            name,
            lang,
            usid,
            uuid,
            mobile,
            color,
            mods,
        } => {
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
        Packet::ClientSnapshot {
            snapshot_id,
            unit_id,
            dead,
            x,
            y,
            pointer_x,
            pointer_y,
            rotation,
            base_rotation,
            x_velocity,
            y_velocity,
            mining_x,
            mining_y,
            boosting,
            shooting,
            chatting,
            building,
            view_x,
            view_y,
            plans,
            view_width,
            view_height,
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
            write_unsigned_short(&mut data, mining_x);
            write_unsigned_short(&mut data, mining_y);
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
        Packet::ConnectCallConfirm => 22,
        Packet::SendChatMessageCall { message } => {
            write_string(&mut data, &message);
            71
        }
        _ => 0,
    };

    let mut buf: Vec<u8> = vec![];
    let length = data.len() as u16 + 4;

    if length > 35 {
        let uncompressed_length = data.len() as u16;
        data = compress(&data, None, false).unwrap();
        let length = data.len() as u16 + 4;

        write_unsigned_short(&mut buf, length);
        buf.push(id);

        write_unsigned_short(&mut buf, uncompressed_length);
        buf.push(0x01);
    } else {
        write_unsigned_short(&mut buf, length);
        buf.push(id);
        write_unsigned_short(&mut buf, data.len() as u16);
        buf.push(0x00);
    }

    buf.extend_from_slice(&data);
    buf
}
