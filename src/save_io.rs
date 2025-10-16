use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use colored::{Color, Colorize};
use serde::Deserialize;
use crate::block_io::readAll;
use crate::type_io::{read_bool, read_byte, read_int, read_short, read_string};

pub fn load_block_types() -> HashMap<String, String> {
  let data = include_str!("data/block_types.json");
  serde_json::from_str(data).unwrap()
}

fn color_from_number(num: u16) -> Color {
  let mut hasher = DefaultHasher::new();
  num.hash(&mut hasher);
  let hash = hasher.finish();
  let color_code = 16 + (hash % 216); // 216 = 6*6*6 cube
  Color::TrueColor {
    r: ((color_code - 16) / 36 * 51) as u8,
    g: (((color_code - 16) / 6 % 6) * 51) as u8,
    b: (((color_code - 16) % 6) * 51) as u8,
  }
}

#[derive(Deserialize)]
struct ContentTypes {
  #[serde(rename="contentTypes")]
  content_types: Vec<String>
}

fn load_content_types() -> Vec<String> {
  let data = include_str!("data/content_types.json");
  serde_json::from_str::<ContentTypes>(data).unwrap().content_types
}

pub fn read_content_header(mut data: &mut Vec<u8>) -> HashMap<String, Vec<String>> {
  let mut result = HashMap::new();
  let content_types = load_content_types();

  let mapped = read_byte(&mut data);
  for _ in 0..mapped {
    let content_type_index = read_byte(&mut data);
    let content_type = content_types.get(content_type_index as usize).unwrap().clone();
    let mut sub_result = vec![];

    let count = read_short(&mut data);
    for _ in 0..count {
      let name = read_string(&mut data).unwrap();
      sub_result.push(name);
    }

    result.insert(content_type, sub_result);
  }

  result
}

#[derive(Clone)]
struct MapTile {
  floor: u16,
  ore: Option<u16>,
  block: Option<u16>,
}

struct Map {
  width: u32,
  height: u32,
  tiles: Vec<Vec<MapTile>>,
}
impl Map {
  fn new(width: u32, height: u32) -> Self {
    let row = vec![
      MapTile {
        floor: 0,
        ore: None,
        block: None,
      };
      width as usize
    ];
    let tiles = vec![row; height as usize];

    Self {
      width,
      height,
      tiles,
    }
  }

  pub fn get(&self, x: u32, y: u32) -> Option<&MapTile> {
    self.tiles.get(y as usize)?.get(x as usize)
  }

  pub fn set_floor(&mut self, x: u32, y: u32, floor: u16) {
    if let Some(tile) = self.tiles.get_mut(y as usize).and_then(|r| r.get_mut(x as usize)) {
      tile.floor = floor;
    }
  }

  pub fn set_ore(&mut self, x: u32, y: u32, ore: u16) {
    if let Some(tile) = self.tiles.get_mut(y as usize).and_then(|r| r.get_mut(x as usize)) {
      tile.ore = Some(ore);
    }
  }

  pub fn set_block(&mut self, x: u32, y: u32, block: u16) {
    if let Some(tile) = self.tiles.get_mut(y as usize).and_then(|r| r.get_mut(x as usize)) {
      tile.block = Some(block);
    }
  }

  pub fn visualize(&self) {
    for y in (0..self.height).rev() {
      for x in 0..self.width {
        if let Some(tile) = self.get(x, y) {
          let color = color_from_number(tile.floor);

          if let Some(block) = tile.block {
            print!("{}", "  ".on_black());
          } else if let Some(ore) = tile.ore {
            let ore_color = color_from_number(ore);
            print!("{}", "::".on_color(color).color(ore_color));
          } else {
            print!("{}", "  ".on_color(color));
          }
        }
      }
      println!();
    }
  }
}

pub fn read_map(mut data: &mut Vec<u8>, content_map: &HashMap<String, Vec<String>>) {
  let width = read_short(&mut data) as u32;
  let height = read_short(&mut data) as u32;
  println!("{width} x {height}");

  let block_types = load_block_types();

  let mut map = Map::new(width, height);

  // Floors and ores
  let mut i = 0;
  while i < (width * height) {
    let x = i % width;
    let y = i / width;
    let floor_id = read_short(&mut data);
    let ore_id = read_short(&mut data);
    let consecutive_count = read_byte(&mut data);
    //if(content.block(floorid) == Blocks.air) floorid = Blocks.stone.id; TODO

    map.set_floor(x, y, floor_id);

    if ore_id != 0 {
      map.set_ore(x, y, ore_id);
    }

    let mut j = i + 1;
    while j < i + 1 + consecutive_count as u32 {
      let new_x = j % width;
      let new_y = j / width;
      map.set_floor(new_x, new_y, floor_id);
      if ore_id != 0 {
        map.set_ore(new_x, new_y, ore_id);
      }

      j += 1;
    }

    i += consecutive_count as u32;
    i += 1;
  }

  // Blocks
  let mut i = 0;
  while i < width * height {
    let x = i % width;
    let y = i / width;

    let block_id = read_short(&mut data);
    //Block block = content.block(stream.readShort());
    //Tile tile = context.tile(i);
    //if(block == null) block = Blocks.air;
    let mut is_center = true;
    let packed_check = read_byte(&mut data);
    let had_entity = (packed_check & 1) != 0;
    let had_data = (packed_check & 4) != 0;

    let mut tile_data = 0;
    let mut floor_data = 0;
    let mut overlay_data = 0;
    let mut extra_data = 0;

    if had_data {
      tile_data = read_byte(&mut data);
      floor_data = read_byte(&mut data);
      overlay_data = read_byte(&mut data);
      extra_data = read_int(&mut data);
    }

    if had_entity {
      is_center = read_bool(&mut data);
    }

    //set block only if this is the center; otherwise, it's handled elsewhere
    if is_center {
      if block_id != 0 {
        map.set_block(x, y, block_id);
      }
    }

    //must be assigned after setBlock, because that can reset data
    if had_data {
      //tile.data = data;
      //tile.floorData = floorData;
      //tile.overlayData = overlayData;
      //tile.extraData = extraData;
      //context.onReadTileData();
    }

    if had_entity {
      if is_center {
        //only read entity for center blocks
        let length = read_short(&mut data);

        let data_length_before = data.len();

        let version = read_byte(&mut data);
        let block_name = content_map.get("block").unwrap().get(block_id as usize).unwrap();
        let block_type = block_types.get(block_name).unwrap();
        let building = readAll(&mut data, block_name.clone(), block_type.clone(), version, content_map);

        let data_read = (data_length_before - data.len()) as u64;
        if data_read != length as u64 {
          panic!("Block parsing failed trying to parse {block_name} ({block_type}) at [{x},{y}]:\nread {data_read} bytes instead of {length}")
        }

        //if(block.hasBuilding()){
         //let length = read_int(&mut data);
        //  try{
        //    readChunkReads(stream, (in, len) -> {
        //      byte revision = in.b();
        //      tile.build.readAll(in, revision);
        //    });
        //  }catch(Throwable e){
        //    throw new IOException("Failed to read tile entity of block: " + block, e);
        //  }
        //} else {
          // skip the entity region, as the entity and its IO code are now gone
          //let length = read_int(&mut data);
          //data.drain(0..length as usize);
        //}

        //context.onReadBuilding();
      }
    } else if !had_data {
      //never read consecutive blocks if there's data
      let consecutive_count = read_byte(&mut data);
      let mut j = i + 1;
      while j < i + 1 + consecutive_count as u32 {
        let new_x = j % width;
        let new_y = j / width;
        if block_id != 0 {
          map.set_block(new_x, new_y, block_id);
        }
        j += 1;
      }

      i += consecutive_count as u32;
    }

    i += 1;
  }

  map.visualize();
}