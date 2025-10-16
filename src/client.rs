use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::{net::{TcpStream, UdpSocket}, sync::mpsc, time, io::AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use crate::packet::{read_packet_tcp, FrameworkPacket, AnyPacket, write_framework_packet, write_packet, Packet, read_packet_udp};
use crate::stream_builder::StreamBuilder;
use crate::type_io::Unit;
use crate::unit_io::{FullUnit, Plan};

pub struct QueuedPacket {
  pub reliable: bool,
  pub packet: Vec<u8>
}

pub struct State {
  pub player_id: u32,
  pub unit: Unit,
  pub x: f32,
  pub y: f32,
  pub chatting: bool,
  pub plans: Vec<Plan>,

  units: HashMap<u32, FullUnit>,
}

pub struct Client {
  pub state: Arc<Mutex<State>>,
  username: String,
  rx_in: mpsc::Receiver<AnyPacket>,
  tx_in: mpsc::Sender<AnyPacket>,
  tx_out: mpsc::Sender<QueuedPacket>,
  streams: HashMap<u32, StreamBuilder>,
  content_map: Arc<RwLock<Option<HashMap<String, Vec<String>>>>>
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
            continue
          },
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
        unit: Unit { unit_type: 0, id: 0 },
        x: 0.0,
        y: 0.0,
        chatting: false,
        plans: vec![],

        units: HashMap::new(),
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

  pub async fn handle_packets(&mut self) -> Option<Packet> {
    while let Some(packet) = self.rx_in.recv().await {
      match packet {
        AnyPacket::Framework(packet) => {
          self.handle_framework_packet(packet).await;
        }
        AnyPacket::Regular(packet) => {
          if let Some(to_user) = self.handle_regular_packet(packet).await {
            return Some(to_user);
          }
        }
      }
    }
    None
  }

  async fn handle_framework_packet(&mut self, packet: FrameworkPacket) {
    match packet {
      FrameworkPacket::KeepAlive => {}
      FrameworkPacket::RegisterTCP(id) => {
        let answer = write_framework_packet(FrameworkPacket::RegisterUDP(id));
        self.queue_out_packet(QueuedPacket {
          reliable: false,
          packet: answer
        }).await;
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
          packet: connect_packet
        }).await;

        let confirm_connect_call_packet = write_packet(Packet::ConnectCallConfirm);
        self.queue_out_packet(QueuedPacket {
          reliable: true,
          packet: confirm_connect_call_packet
        }).await;

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
              rotation: (i %360 * 50) as f32,
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

            tx_out_keepalive.send(QueuedPacket {
              reliable: true,
              packet: write_packet(snapshot)
            }).await.unwrap();

            if i % (5 * 5) == 0 {
              tx_out_keepalive.send(QueuedPacket {
                reliable: true,
                packet: write_framework_packet(FrameworkPacket::KeepAlive)
              }).await.unwrap();
            }
            if i % (15 * 5) == 0 {
              tx_out_keepalive.send(QueuedPacket {
                reliable: false,
                packet: write_framework_packet(FrameworkPacket::KeepAlive)
              }).await.unwrap();
            }
          }
        });
      }
      _ => println!("fw> {packet:?}")
    }
  }

  async fn handle_regular_packet(&mut self, packet: Packet) -> Option<Packet> {
    match packet {
      Packet::StreamBegin { id, stream_type, total} => {
        self.streams.insert(id, StreamBuilder::new(id, stream_type, total));
        None
      }
      Packet::StreamChunk { id, data } => {
        let stream = match self.streams.get_mut(&id) {
          Some(stream) => stream,
          None => {
            println!("Stream {id} not found!");
            return None;
          }
        };
        stream.add(data);
        if stream.is_done() {
          let stream = self.streams.remove(&id).unwrap();
          let content = self.content_map.read().await;
          let packet = stream.build(&content).unwrap();
          self.queue_in_packet(AnyPacket::Regular(packet)).await;
        }
        None
      }
      Packet::WorldStream { id, wave, wave_time, tick, seed0 , seed1, content_map: content } => {
        let mut current_state = self.state.lock().await;
        current_state.player_id = id;
        println!("Set player id {id}");

        {
          let mut content_map = self.content_map.write().await;
          *content_map = Some(content.clone());
        }
        
        Some(Packet::WorldStream { id, wave, wave_time, tick, seed0, seed1, content_map: content })
        //None
      }
      Packet::EntitySnapshot { units } => {
        let mut current_state = self.state.lock().await;

        match current_state.units.get(&current_state.player_id) {
          Some(unit) => match unit {
            FullUnit::Player { unit, .. } => {
              current_state.unit = unit.clone();
            },
            _ => unreachable!(),
          }
          None => {}
        };

        current_state.units = units;
        None
      }
      Packet::KickCall { reason } => {
        println!("Kicked: {reason}");
        None
      },
      Packet::KickCall2 { reason } => {
        println!("Kicked: {reason:?}");
        None
      },
      Packet::SpawnCall { tile_x, tile_y, entity } => {
        println!("Spawn call: {tile_x}/{tile_y} {entity}");
        let mut current_state = self.state.lock().await;
        println!("{}", current_state.player_id);
        if current_state.player_id == entity {
          current_state.x = (tile_x * 8) as f32;
          current_state.y = (tile_y * 8) as f32;
        }
        None
      },
      Packet::SendMessageCall2 { message, unformatted, sender } => 
        Some(Packet::SendMessageCall2 {message, unformatted, sender}),
      //Packet::Other(id) => println!(">> {id}"),
      _ => None,
    }
  }
}
