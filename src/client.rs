use crate::block_io::{read_block, BaseBlockData, Block};
use crate::packet::{
    AnyPacket, FrameworkPacket, Packet, read_packet_tcp, read_packet_udp, write_framework_packet,
    write_packet,
};
use crate::save_io::{Map, load_block_types};
use crate::stream_builder::StreamBuilder;
use crate::type_io::{Reader, Tile, Unit, read_tile};
use crate::unit_io::{FullUnit, Plan};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, UdpSocket},
    sync::mpsc,
    time,
};

pub struct QueuedPacket {
    pub reliable: bool,
    pub packet: Vec<u8>,
}

pub struct State {
    pub player_id: u32,
    pub unit: Unit,
    pub x: f32,
    pub y: f32,
    pub chatting: bool,
    pub plans: Vec<Plan>,

    pub units: HashMap<u32, FullUnit>,
    pub map: Map,
}

pub struct Client {
    pub state: Arc<Mutex<State>>,
    username: String,
    rx_in: mpsc::Receiver<AnyPacket>,
    tx_in: mpsc::Sender<AnyPacket>,
    tx_out: mpsc::Sender<QueuedPacket>,
    streams: HashMap<u32, StreamBuilder>,
    content_map: Arc<RwLock<Option<HashMap<String, Vec<String>>>>>,
}

// TODO improve included data
pub enum ClientEvent {
    MapLoaded,
    BlockChanged {
        tile: Tile,
    },
    UnitSnapshot,
    ChatMessage {
        message: String,
        unformatted: Option<String>,
        sender: u32,
    },
}

impl Client {
    pub async fn new(ip: String, username: String) -> Client {
        let (tx_in, rx_in) = mpsc::channel::<AnyPacket>(100);
        let (tx_out, mut rx_out) = mpsc::channel::<QueuedPacket>(100);

        let tcp = TcpStream::connect(&ip).await.unwrap();
        let (mut tcp_read, mut tcp_write) = tcp.into_split();

        let udp = Arc::from(UdpSocket::bind("0.0.0.0:0").await.unwrap());
        let mut udp_read = udp.clone();
        udp.connect(ip).await.unwrap();

        let default_content_map_path = PathBuf::from("content-map.json");
        let default_content_map = if default_content_map_path.exists() {
            let default_content_map_data = fs::read_to_string(default_content_map_path).unwrap();
            Some(serde_json::from_str(&default_content_map_data).unwrap())
        } else {
            None
        };

        let content_map = Arc::new(RwLock::new(default_content_map));

        // TCP Read
        let tx_in_tcp = tx_in.clone();
        let content = Arc::clone(&content_map);
        tokio::spawn(async move {
            loop {
                let content_map = content.read().await;
                let packet = read_packet_tcp(&mut tcp_read, &content_map).await.unwrap();
                tx_in_tcp.send(packet).await.unwrap();
            }
        });

        // UDP Read
        let tx_in_udp = tx_in.clone();
        let content = Arc::clone(&content_map);
        tokio::spawn(async move {
            loop {
                let map = content.read().await;
                let packet = match read_packet_udp(&mut udp_read, &map).await {
                    Ok(packet) => packet,
                    Err(err) => {
                        println!("[UDP Error] {err:?}");
                        continue;
                    }
                };
                tx_in_udp.send(packet).await.unwrap();
            }
        });

        // Send
        tokio::spawn(async move {
            while let Some(packet) = rx_out.recv().await {
                match packet.reliable {
                    true => {
                        tcp_write.write(&packet.packet).await.unwrap();
                    }
                    false => {
                        udp.send(&packet.packet).await.unwrap();
                    }
                }
            }
        });

        Client {
            state: Arc::new(Mutex::new(State {
                player_id: 0,
                unit: Unit {
                    unit_type: 0,
                    id: 0,
                },
                x: -1.0,
                y: -1.0,
                chatting: false,
                plans: vec![],

                units: HashMap::new(),
                map: Map::new(0, 0),
            })),
            username,
            rx_in,
            tx_in,
            tx_out,
            streams: HashMap::new(),
            content_map: content_map.clone(),
        }
    }

    pub async fn queue_out_packet(&self, packet: QueuedPacket) {
        self.tx_out.send(packet).await.unwrap()
    }

    async fn queue_in_packet(&self, packet: AnyPacket) {
        self.tx_in.send(packet).await.unwrap();
    }

    pub async fn handle_packets(&mut self, channel: mpsc::Sender<ClientEvent>) {
        while let Some(packet) = self.rx_in.recv().await {
            match packet {
                AnyPacket::Framework(packet) => {
                    self.handle_framework_packet(packet).await;
                }
                AnyPacket::Regular(packet) => {
                    self.handle_regular_packet(packet, &channel).await;
                }
            }
        }
    }

    async fn handle_framework_packet(&mut self, packet: FrameworkPacket) {
        match packet {
            FrameworkPacket::KeepAlive => {}
            FrameworkPacket::RegisterTCP(id) => {
                let answer = write_framework_packet(FrameworkPacket::RegisterUDP(id));
                self.queue_out_packet(QueuedPacket {
                    reliable: false,
                    packet: answer,
                })
                .await;
            }
            FrameworkPacket::RegisterUDP(..) => {
                let connect_packet = write_packet(Packet::Connect {
                    version: 146,
                    client: "official".to_string(),
                    name: self.username.clone(),
                    lang: "en".to_string(),
                    usid: "USIGAAAAAAA=".to_string(),
                    uuid: "UUIGAAAAAAA=".to_string(),
                    mobile: false,
                    color: vec![0xff, 0xa1, 0x08, 0xff],
                    mods: vec![],
                });
                self.queue_out_packet(QueuedPacket {
                    reliable: true,
                    packet: connect_packet,
                })
                .await;

                let confirm_connect_call_packet = write_packet(Packet::ConnectCallConfirm);
                self.queue_out_packet(QueuedPacket {
                    reliable: true,
                    packet: confirm_connect_call_packet,
                })
                .await;

                // KeepAlive & State
                let tx_out_keepalive = self.tx_out.clone();
                let state_keepalive = Arc::clone(&self.state);
                tokio::spawn(async move {
                    let mut i = 0;
                    let mut interval = time::interval(Duration::from_millis(200));
                    loop {
                        interval.tick().await;
                        i += 1;

                        let current_state = state_keepalive.lock().await;

                        let snapshot = Packet::ClientSnapshot {
                            snapshot_id: i,
                            unit_id: current_state.unit.id,
                            dead: false,
                            x: current_state.x,
                            y: current_state.y,
                            pointer_x: current_state.x,
                            pointer_y: current_state.y,
                            rotation: (i % 360 * 50) as f32,
                            base_rotation: 0.0,
                            x_velocity: 0.0,
                            y_velocity: 0.0,
                            mining_x: 0,
                            mining_y: 0,
                            boosting: false,
                            shooting: false,
                            chatting: current_state.chatting,
                            building: true,
                            plans: current_state.plans.clone(),
                            view_x: 0.0,
                            view_y: 0.0,
                            view_width: 1920.0,
                            view_height: 1080.0,
                        };

                        tx_out_keepalive
                            .send(QueuedPacket {
                                reliable: true,
                                packet: write_packet(snapshot),
                            })
                            .await
                            .unwrap();

                        if i % (5 * 5) == 0 {
                            tx_out_keepalive
                                .send(QueuedPacket {
                                    reliable: true,
                                    packet: write_framework_packet(FrameworkPacket::KeepAlive),
                                })
                                .await
                                .unwrap();
                        }
                        if i % (15 * 5) == 0 {
                            tx_out_keepalive
                                .send(QueuedPacket {
                                    reliable: false,
                                    packet: write_framework_packet(FrameworkPacket::KeepAlive),
                                })
                                .await
                                .unwrap();
                        }
                    }
                });
            }
            _ => println!("fw> {packet:?}"),
        }
    }

    async fn handle_regular_packet(&mut self, packet: Packet, sender: &mpsc::Sender<ClientEvent>) {
        match packet {
            Packet::StreamBegin {
                id,
                stream_type,
                total,
            } => {
                self.streams
                    .insert(id, StreamBuilder::new(id, stream_type, total));
            }
            Packet::StreamChunk { id, data } => {
                let stream = match self.streams.get_mut(&id) {
                    Some(stream) => stream,
                    None => {
                        eprintln!("Stream {id} not found!");
                        return;
                    }
                };
                stream.add(data);
                if stream.is_done() {
                    let stream = self.streams.remove(&id).unwrap();
                    let content = self.content_map.read().await;
                    let packet = stream.build(&content).unwrap();
                    self.queue_in_packet(AnyPacket::Regular(packet)).await;
                }
            }
            Packet::WorldStream {
                id,
                content_map: content,
                map,
                ..
            } => {
                let mut current_state = self.state.lock().await;
                current_state.player_id = id;
                current_state.map = map;

                {
                    let mut content_map = self.content_map.write().await;
                    *content_map = Some(content.clone());
                }

                sender.send(ClientEvent::MapLoaded).await.unwrap();
            }
            Packet::BeginPlace { x, y, rotation, result, team, .. } => {
                let mut state = self.state.lock().await;
                let map_tile = match state.map.get_mut(x, y) {
                    Some(map_tile) => map_tile,
                    None => {
                        eprintln!("Recvd begin place before map was loaded, ignoring"); 
                        return;
                    }
                };
                map_tile.block_id = Some(result as i16);
                // TODO improve
                map_tile.block = Some(Block {
                    block_type: "Construct".to_string(),
                    name: "Construct".to_string(),
                    base: BaseBlockData {
                        team,
                        rotation: rotation as u8,
                        version: 0,
                        legacy: false,
                        items: None,
                        liquids: None,
                        power: None,
                        on: None,
                        module_bitmask: 0,
                        health: 1f32,
                    },
                    specific: None
                })
            }
            Packet::ConstructFinish { tile, block, .. } => {
                let mut state = self.state.lock().await;
                let map_tile = state.map.get_mut(tile.x as u32, tile.y as u32).unwrap();
                map_tile.block_id = Some(block);

                let block_types = load_block_types();
                let content_map = match self.content_map.read().await.clone() {
                    Some(map ) => map,
                    None => {
                        // TODO
                        return;
                    }
                };

                let block_name = content_map
                    .get("block")
                    .unwrap()
                    .get(block as usize)
                    .unwrap();
                let block_type = block_types.get(block_name).unwrap();

                if let Some(block) = &mut map_tile.block {
                    block.block_type = block_type.clone();
                    block.name = block_name.clone();
                    // TODO update config
                } else {
                    eprintln!("Construct block at {tile:?} missing!");
                }

                sender
                    .send(ClientEvent::BlockChanged { tile })
                    .await
                    .unwrap();
            }
            Packet::DeconstructFinish { tile, .. } => {
                let mut state = self.state.lock().await;
                let map_tile = state.map.get_mut(tile.x as u32, tile.y as u32).unwrap();
                map_tile.block_id = None;
                map_tile.block = None;
                sender.send(ClientEvent::BlockChanged { tile }).await.unwrap();
            }
            // TODO Broken
            Packet::BlockSnapshot { amount, data } => {
                return;
                
                let mut reader = Reader::new(data);
                let mut state = self.state.lock().await;

                for _ in 0..amount {
                    let tile = read_tile(&mut reader);
                    let block_id = reader.short();

                    let map_tile = match state.map.get_mut(tile.x as u32, tile.y as u32){
                        Some(map_tile) => map_tile,
                        None => {
                            eprintln!("Invalid state: Block snapshot contains locally missing block at {tile:?}");
                            return;
                        }
                    };
                    if map_tile.block_id.unwrap() != block_id {
                        eprintln!(
                            "Invalid block id at {tile:?}: Expected {block_id} but found {:?}",
                            map_tile.block_id
                        );
                    }

                    let block_types = load_block_types();
                    let content = self.content_map.read().await.clone().unwrap();

                    let block_name = content
                        .get("block")
                        .unwrap()
                        .get(block_id as usize)
                        .unwrap();
                    let block_type = block_types.get(block_name).unwrap();

                    let content_map = self.content_map.read().await;
                    let block = read_block(
                        &mut reader,
                        block_name.clone(),
                        block_type.clone(),
                        map_tile.block.clone().unwrap().base.version,
                        &content_map.clone().unwrap(),
                    );

                    map_tile.block = Some(block);

                    sender
                        .send(ClientEvent::BlockChanged { tile })
                        .await
                        .unwrap();
                }
            }
            Packet::EntitySnapshot { units } => {
                let mut current_state = self.state.lock().await;
                let possible_unit = current_state.units.get(&current_state.player_id).cloned();

                match possible_unit {
                    Some(unit) => match unit {
                        FullUnit::Player { unit, x, y, .. } => {
                            // TODO pos changed event?
                            if current_state.x == -1.0 {
                                current_state.x = x;
                                println!("Update x: {}", x);
                            }
                            if current_state.y == -1.0 {
                                current_state.y = y;
                                println!("Update y: {}", y);
                            }

                            current_state.unit = unit.clone();
                        }
                        _ => unreachable!(),
                    },
                    None => {}
                };

                current_state.units = units;

                sender.send(ClientEvent::UnitSnapshot).await.unwrap();
            }
            Packet::KickCall { reason } => {
                eprintln!("Kicked: {reason}");
            }
            Packet::KickCall2 { reason } => {
                eprintln!("Kicked: {reason:?}");
            }
            Packet::SpawnCall {
                tile_x,
                tile_y,
                entity,
            } => {
                // TODO player spawned event?
                println!("Spawn call: {tile_x}/{tile_y} {entity}");
                let mut current_state = self.state.lock().await;
                println!("{}", current_state.player_id);
                if current_state.player_id == entity {
                    // TODO pos change event?
                    println!("Set coords");
                    current_state.x = (tile_x * 8) as f32;
                    current_state.y = (tile_y * 8) as f32;
                }
            }
            Packet::RotateBlockCall { tile, rotation, .. } => {
                let mut state = self.state.lock().await;
                let map_tile = state.map.get_mut(tile.x as u32, tile.y as u32).unwrap();
                map_tile.block.as_mut().unwrap().base.rotation = rotation;

                sender
                    .send(ClientEvent::BlockChanged { tile })
                    .await
                    .unwrap();
            }
            Packet::SendMessageCall2 {
                message,
                unformatted,
                sender: author,
            } => {
                sender
                    .send(ClientEvent::ChatMessage {
                        message,
                        unformatted,
                        sender: author,
                    })
                    .await
                    .unwrap();
            }
            Packet::Other(id) => {
                eprintln!("Unhandled packet: {id}");
            }
            _ => {}
        }
    }
}
