use std::collections::HashMap;
use phf::phf_map;
use crate::block_io::readAll;
use crate::save_io::load_block_types;
use crate::type_io::{read_items, read_object, read_prefixed_string, read_tile, read_unit, read_vec2, write_byte, write_int, write_object, write_short, write_tile, Items, Object, Reader, Tile, Unit, Vec2};

static UNIT_MAP: phf::Map<u8, &'static str> = phf_map! {
    0u8 => "UnitEntity",
    2u8 => "BlockUnitUnit",
    3u8 => "UnitEntity",
    4u8 => "MechUnit",
    5u8 => "PayloadUnit",
    6u8 => "",
    7u8 => "",
    8u8 => "",
    9u8 => "",
    10u8 => "Fire",
    11u8 => "",
    12u8 => "Player",
    13u8 => "Puddle",
    14u8 => "WeatherState",
    15u8 => "",
    16u8 => "UnitEntity",
    17u8 => "MechUnit",
    18u8 => "UnitEntity",
    19u8 => "MechUnit",
    20u8 => "UnitWaterMove",
    21u8 => "LegsUnit",
    24u8 => "LegsUnit",
    26u8 => "PayloadUnit",
    28u8 => "",
    29u8 => "LegsUnit",
    30u8 => "UnitEntity",
    31u8 => "UnitEntity",
    32u8 => "MechUnit",
    33u8 => "LegsUnit",
    35u8 => "WorldLabel",
    36u8 => "BuildingTetherPayloadUnit",
    39u8 => "TimedKillUnit",
    42u8 => "",
    43u8 => "TankUnit",
    45u8 => "ElevationMoveUnit",
    46u8 => "CrawlUnit",
};

pub fn read_abilities(reader: &mut Reader) -> Vec<f32> {
  let length = reader.byte();
  let mut abilities = vec![];
  for _ in 0..length {
    let data = reader.float();
    abilities.push(data);
  }
  abilities
}


#[derive(Debug, Clone)]
pub struct Plan {
  pub plan_type: u8, // TODO this might be a boolean for deconstruction
  pub position: Tile,
  pub block: Option<i16>,
  pub rotation: Option<u8>,
  pub has_config: Option<bool>,
  pub config: Option<Object>
}

pub fn read_plan(reader: &mut Reader) -> Plan {
  let plan_type = reader.byte();
  let position = read_tile(reader);

  if plan_type == 1 {
    Plan {
      plan_type,
      position,
      block: None,
      rotation: None,
      has_config: None,
      config: None
    }
  } else {
    let block = reader.short();
    let rotation = reader.byte();
    let has_config = reader.byte() != 0;
    let config = read_object(reader);
    Plan {
      plan_type,
      position,
      block: Some(block),
      rotation: Some(rotation),
      has_config: Some(has_config),
      config: Some(config)
    }
  }
}

pub fn read_plans(reader: &mut Reader) -> Vec<Plan> {
  let mut plans = vec![];
  let plan_count = reader.short();
  for _ in 0..plan_count {
    plans.push(read_plan(reader));
  }
  plans
}

pub fn read_plans_queue(reader: &mut Reader) -> Vec<Plan> {
  let mut plans = vec![];
  let plan_count = reader.int();
  for _ in 0..plan_count {
    plans.push(read_plan(reader));
  }
  plans
}

pub fn write_plan(buf: &mut Vec<u8>, plan: Plan) {
  write_byte(buf, plan.plan_type);
  write_tile(buf, plan.position);

  if plan.plan_type == 0 {
    write_short(buf, plan.block.unwrap());
    write_byte(buf, plan.rotation.unwrap());
    write_byte(buf, plan.has_config.unwrap() as u8);
    write_object(buf, plan.config.unwrap());
  }
}

pub fn write_plans(buf: &mut Vec<u8>, plans: Vec<Plan>) {
  write_int(buf, plans.len() as u32);
  for plan in plans {
    write_plan(buf, plan);
  }
}

#[derive(Debug, Clone)]
pub struct Status {
  id: i16,
  time: f32,
}

pub fn read_status(reader: &mut Reader) -> Status {
  let id = reader.short();
  let time = reader.float();
  Status { id, time }
}

pub fn read_statuses(reader: &mut Reader) -> Vec<Status> {
  let mut statuses = vec![];
  let status_count = reader.int();
  for _ in 0..status_count {
    let status = read_status(reader);
    statuses.push(status);
  }
  statuses
}

#[derive(Debug, Clone)]
pub struct Mount {
  state: u8,
  x: f32,
  y: f32
}

pub fn read_mounts(reader: &mut Reader) -> Vec<Mount> {
  let mut mounts = vec![];
  let amount = reader.byte() as usize;
  for _ in 0..amount {
    let state = reader.byte();
    let x = reader.float();
    let y = reader.float();
    mounts.push(Mount { state, x, y });
  }
  mounts
}


// TODO
#[derive(Debug, Clone)]
pub struct Controller {}

// TODO
pub fn read_controller(reader: &mut Reader) -> Controller {
  let controller_type = reader.byte();

  match controller_type {
    0 => {
      reader.int();
    },
    1 => {
      reader.bytes(4);
    },
    3 => {
      reader.int();
    }
    4 | 6 | 7 | 8 => {
      let has_attack = reader.byte() != 0;
      let has_pos = reader.byte() != 0;

      let mut x = None;
      let mut y = None;
      if has_pos {
        x = Some(reader.float());
        y = Some(reader.float());
      }

      let mut entity_type = None;
      let mut attack = None;
      if has_attack {
        entity_type = Some(reader.byte());
        attack = Some(reader.int());
      }

      let mut id = None;
      if controller_type == 6 || controller_type == 7 || controller_type == 8 {
        id = Some(reader.byte());
      }

      let mut attack_info_build = None;
      let mut attack_info_unit = None;
      let mut attack_info_vec_x = None;
      let mut attack_info_vec_y = None;
      let mut final_controller_type = vec![];
      if controller_type == 7 ||controller_type == 8 {
        let length = reader.byte();
        for _ in 0..length {
          let controller_type2 = reader.byte();
          final_controller_type.push(controller_type2);

          if controller_type2 == 0 {
            attack_info_build = Some(reader.int());
          } else if controller_type2 == 1 {
            attack_info_unit = Some(reader.int());
          } else if controller_type2 == 2 {
            attack_info_vec_x = Some(reader.float());
            attack_info_vec_y = Some(reader.float());
          }
        }
      }

      let mut stance = None;
      if controller_type == 8 {
        let byte = reader.byte();
        if byte == 255 {
          stance = Some(None)
        } else {
          stance = Some(Some(byte))
        }
      }

      /*Controller {
        entity_type,
        length2,
        ctype,
        has_attack,
        has_pos,
        pos,
        attackinfo,
        attack,
        id,
        stance
      }*/
    }
    _ => {}
  };
  Controller {}
}

// TODO
#[derive(Debug, Clone)]
pub struct Payload {}

// TODO
pub fn read_payload(reader: &mut Reader, content_map: &HashMap<String, Vec<String>>) -> Option<Payload> {
  let ex = reader.bool();
  if !ex {
    return None
  }
  
  let payload_type = reader.byte();
  if payload_type == 1 {
    let block_types = load_block_types();
    
    let id = reader.short();
    let version = reader.byte();
    let block_name = content_map.get("block").unwrap().get(id as usize).unwrap();
    let block_type = block_types.get(block_name).unwrap();
    let block = readAll(reader, block_name.clone(), block_type.clone(), version, content_map);
    //return [id, block]
  } else {
    let unit = read_unit(reader);
    //return unit
  }
  
  Some(Payload {})
}

pub fn read_payloads(reader: &mut Reader, content_map: &HashMap<String, Vec<String>>) -> Vec<Payload> {
  let mut payloads = vec![];

  let amount = reader.int();
  for _ in 0..amount {
    let payload = read_payload(reader, content_map).unwrap();
    payloads.push(payload)
  }

  payloads
}


// TODO
#[derive(Debug, Clone)]
pub enum FullUnit {
  GenericUnit {
    revision: Option<i16>,
    abilities: Vec<f32>,
    ammo: f32,
    building: Option<u32>,
    base_rotation:  Option<f32>,
    controller: Controller,
    elevation: f32,
    flag: f64,
    health: f32,
    shooting: bool,
    lifetime: Option<f32>,
    mining_position: Tile,
    mounts: Vec<Mount>,
    payloads: Option<Vec<Payload>>,
    plans: Vec<Plan>,
    rotation: f32,
    shield: f32,
    spawned_by_core: bool,
    items: Items,
    statuses: Vec<Status>,
    team: u8,
    time: Option<f32>,
    unit_type: i16, // TODO check what 'utype' really is
    upgrade_building: u8, // TODO check what 'upgbuilding' really is
    velocity: Vec2,
    x: f32,
    y: f32
  },
  Fire {
    revision: Option<i16>,
    lifetime: f32,
    tile: Tile,
    time: f32,
    x: f32,
    y: f32,
  },
  Puddle {
    revision: Option<i16>,
    amount: f32,
    liquid: i16,
    tile: Tile,
    x: f32,
    y: f32,
  },
  Player {
    revision: Option<i16>,
    admin: bool,
    boosting: bool,
    color: u32,
    mouse_x: f32,
    mouse_y: f32,
    name: String,
    shooting: bool,
    team: u8,
    typing: bool,
    unit: Unit,
    x: f32,
    y: f32,
  },
  WeatherState {
    revision: Option<i16>,
    effect: f32,
    intensity: f32,
    life: f32,
    opacity: f32,
    weather: i16,
    wind_x: f32,
    wind_y: f32,
  },
  WorldLabel {
    revision: Option<i16>,
    flags: u8,
    fonts: f32,
    str: String,
    x: f32,
    y: f32,
  },
  Unknown
}

pub fn read_full_unit(reader: &mut Reader, type_id: u8, has_revision: bool, content_map: &Option<HashMap<String, Vec<String>>>) -> FullUnit {
  let mut revision = None;
  if has_revision {
    revision = Some(reader.short());
  }

  let unit_type = UNIT_MAP.get(&type_id).unwrap();

  if unit_type == &"MechUnit" || unit_type == &"CrawlUnit" || unit_type == &"ElevationMoveUnit" ||
      unit_type == &"TankUnit" || unit_type == &"UnitEntity" || unit_type == &"BlockUnitUnit" ||
      unit_type == &"UnitWaterMove" || unit_type == &"LegsUnit" || unit_type == &"TimedKillUnit" ||
      unit_type == &"PayloadUnit" || unit_type == &"BuildingTetherPayloadUnit" {
    let abilities = read_abilities(reader);
    let ammo = reader.float();

    let mut building = None;
    if unit_type == &"BuildingTetherPayloadUnit" {
      building = Some(reader.int());
    }

    let mut base_rotation = None;
    if unit_type == &"MechUnit" {
      base_rotation = Some(reader.float());
    }

    let controller = read_controller(reader);
    let elevation = reader.float(); // TODO check if 'elv' really is elevation
    let flag = reader.double();
    let health = reader.float();
    let shooting = reader.byte() != 0;

    let mut lifetime = None;
    if unit_type == &"TimedKillUnit" {
      lifetime = Some(reader.float());
    }

    let mining_position = read_tile(reader);
    let mounts = read_mounts(reader);

    let mut payloads = None;
    if unit_type == &"PayloadUnit" || unit_type == &"BuildingTetherPayloadUnit" {
      payloads = Some(read_payloads(reader, &content_map.clone().expect("Received unit data before content map was set and no default map is present")));
    }

    let plans = read_plans_queue(reader);
    let rotation = reader.float();
    let shield = reader.float();
    let spawned_by_core = reader.byte() != 0; // TODO check if 'spbycore' really is spawned_by_core
    let items = read_items(reader);
    let statuses = read_statuses(reader);
    let team = reader.byte();

    let mut time = None;
    if unit_type == &"TimedKillUnit" {
      time = Some(reader.float());
    }

    let unit_type = reader.short(); // TODO check if this is what 'utype' actually is
    let upgrade_building = reader.byte(); // TODO check what 'updbuilding' actually is
    let velocity = read_vec2(reader);
    let x = reader.float();
    let y = reader.float();

    return FullUnit::GenericUnit {
      revision,
      abilities,
      ammo,
      building,
      base_rotation,
      controller,
      elevation,
      flag,
      health,
      shooting,
      lifetime,
      mining_position,
      mounts,
      payloads,
      plans,
      rotation,
      shield,
      spawned_by_core,
      items,
      statuses,
      team,
      time,
      unit_type,
      upgrade_building,
      velocity,
      x,
      y
    }
  } else if unit_type == &"Fire" {
    return FullUnit::Fire {
      revision,
      lifetime: reader.float(),
      tile: read_tile(reader),
      time: reader.float(),
      x: reader.float(),
      y: reader.float(),
    };
  } else if unit_type == &"Puddle" {
    return FullUnit::Puddle {
      revision,
      amount: reader.float(),
      liquid: reader.short(),
      tile: read_tile(reader),
      x: reader.float(),
      y: reader.float(),
    };
  } else if unit_type == &"Player" {
    return FullUnit::Player {
      revision,
      admin: reader.byte() != 0,
      boosting: reader.byte() != 0,
      color: reader.int(),
      mouse_x: reader.float(),
      mouse_y: reader.float(),
      name: read_prefixed_string(reader).unwrap(),
      shooting: reader.byte() != 0,
      team: reader.byte(),
      typing: reader.byte() != 0,
      unit: read_unit(reader),
      x: reader.float(),
      y: reader.float(),
    };
  } else if unit_type == &"WeatherState" {
    return FullUnit::WeatherState {
      revision,
      effect: reader.float(),
      intensity: reader.float(),
      life: reader.float(),
      opacity: reader.float(),
      weather: reader.short(),
      wind_x: reader.float(),
      wind_y: reader.float(),
    };
  } else if unit_type == &"WorldLabel" {
    return FullUnit::WorldLabel {
      revision,
      flags: reader.byte(),
      fonts: reader.float(),
      str: read_prefixed_string(reader).unwrap(),
      x: reader.float(),
      y: reader.float(),
    };
  }

  FullUnit::Unknown
}