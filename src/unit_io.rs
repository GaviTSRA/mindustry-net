use phf::phf_map;
use crate::type_io::{read_byte, read_double, read_float, read_int, read_items, read_object, read_prefixed_string, read_short, read_tile, read_unit, read_vec2, write_byte, write_int, write_object, write_short, write_tile, Items, Object, Tile, Unit, Vec2};

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

pub fn read_abilities(buf: &mut Vec<u8>) -> Vec<f32> {
  let length = read_byte(buf);
  let mut abilities = vec![];
  for _ in 0..length {
    let data = read_float(buf);
    abilities.push(data);
  }
  abilities
}


#[derive(Debug, Clone)]
pub struct Plan {
  pub plan_type: u8, // TODO this might be a boolean for deconstruction
  pub position: Tile,
  pub block: Option<u16>,
  pub rotation: Option<u8>,
  pub has_config: Option<bool>,
  pub config: Option<Object>
}

pub fn read_plan(buf: &mut Vec<u8>) -> Plan {
  let plan_type = read_byte(buf);
  let position = read_tile(buf);

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
    let block = read_short(buf);
    let rotation = read_byte(buf);
    let has_config = read_byte(buf) != 0;
    let config = read_object(buf);
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

pub fn read_plans(buf: &mut Vec<u8>) -> Vec<Plan> {
  let mut plans = vec![];
  let plan_count = read_int(buf);
  for _ in 0..plan_count {
    plans.push(read_plan(buf));
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

#[derive(Debug)]
pub struct Status {
  id: u16,
  time: f32,
}

pub fn read_status(buf: &mut Vec<u8>) -> Status {
  let id = read_short(buf);
  let time = read_float(buf);
  Status { id, time }
}

pub fn read_statuses(buf: &mut Vec<u8>) -> Vec<Status> {
  let mut statuses = vec![];
  let status_count = read_int(buf);
  for _ in 0..status_count {
    let status = read_status(buf);
    statuses.push(status);
  }
  statuses
}

#[derive(Debug)]
pub struct Mount {
  state: u8,
  x: f32,
  y: f32
}

pub fn read_mounts(buf: &mut Vec<u8>) -> Vec<Mount> {
  let mut mounts = vec![];
  let amount = read_byte(buf) as usize;
  for _ in 0..amount {
    let state = read_byte(buf);
    let x = read_float(buf);
    let y = read_float(buf);
    mounts.push(Mount { state, x, y });
  }
  mounts
}


// TODO
#[derive(Debug)]
pub struct Controller {}

// TODO
pub fn read_controller(buf: &mut Vec<u8>) -> Controller {
  let controller_type = read_byte(buf);

  match controller_type {
    0 => {
      read_int(buf);
    },
    1 => {
      buf.drain(..4);
    },
    3 => {
      read_int(buf);
    }
    4 | 6 | 7 | 8 => {
      let has_attack = read_byte(buf) != 0;
      let has_pos = read_byte(buf) != 0;

      let mut x = None;
      let mut y = None;
      if has_pos {
        x = Some(read_float(buf));
        y = Some(read_float(buf));
      }

      let mut entity_type = None;
      let mut attack = None;
      if has_attack {
        entity_type = Some(read_byte(buf));
        attack = Some(read_int(buf));
      }

      let mut id = None;
      if controller_type == 6 || controller_type == 7 || controller_type == 8 {
        id = Some(read_byte(buf));
      }

      let mut attack_info_build = None;
      let mut attack_info_unit = None;
      let mut attack_info_vec_x = None;
      let mut attack_info_vec_y = None;
      let mut final_controller_type = vec![];
      if controller_type == 7 ||controller_type == 8 {
        let length = read_byte(buf);
        for _ in 0..length {
          let controller_type2 = read_byte(buf);
          final_controller_type.push(controller_type2);

          if controller_type2 == 0 {
            attack_info_build = Some(read_int(buf));
          } else if controller_type2 == 1 {
            attack_info_unit = Some(read_int(buf));
          } else if controller_type2 == 2 {
            attack_info_vec_x = Some(read_float(buf));
            attack_info_vec_y = Some(read_float(buf));
          }
        }
      }

      let mut stance = None;
      if controller_type == 8 {
        let byte = read_byte(buf);
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
#[derive(Debug)]
pub struct Payload {}

// TODO
pub fn read_payload(buf: &mut Vec<u8>) -> Payload {
  Payload {}
}

pub fn read_payloads(buf: &mut Vec<u8>) -> Vec<Payload> {
  let mut payloads = vec![];

  let amount = read_int(buf);
  for _ in 0..amount {
    let payload = read_payload(buf);
    payloads.push(payload)
  }

  payloads
}


// TODO
#[derive(Debug)]
pub enum FullUnit {
  GenericUnit {
    revision: Option<u16>,
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
    unit_type: u16, // TODO check what 'utype' really is
    upgrade_building: u8, // TODO check what 'upgbuilding' really is
    velocity: Vec2,
    x: f32,
    y: f32
  },
  Fire {
    revision: Option<u16>,
    lifetime: f32,
    tile: Tile,
    time: f32,
    x: f32,
    y: f32,
  },
  Puddle {
    revision: Option<u16>,
    amount: f32,
    liquid: u16,
    tile: Tile,
    x: f32,
    y: f32,
  },
  Player {
    revision: Option<u16>,
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
    revision: Option<u16>,
    effect: f32,
    intensity: f32,
    life: f32,
    opacity: f32,
    weather: u16,
    wind_x: f32,
    wind_y: f32,
  },
  WorldLabel {
    revision: Option<u16>,
    flags: u8,
    fonts: f32,
    str: String,
    x: f32,
    y: f32,
  },
  Unknown
}

pub fn read_full_unit(buf: &mut Vec<u8>, type_id: u8, has_revision: bool) -> FullUnit {
  let mut revision = None;
  if has_revision {
    revision = Some(read_short(buf));
  }

  let unit_type = UNIT_MAP.get(&type_id).unwrap();

  if unit_type == &"MechUnit" || unit_type == &"CrawlUnit" || unit_type == &"ElevationMoveUnit" ||
      unit_type == &"TankUnit" || unit_type == &"UnitEntity" || unit_type == &"BlockUnitUnit" ||
      unit_type == &"UnitWaterMove" || unit_type == &"LegsUnit" || unit_type == &"TimedKillUnit" ||
      unit_type == &"PayloadUnit" || unit_type == &"BuildingTetherPayloadUnit" {
    let abilities = read_abilities(buf);
    let ammo = read_float(buf);

    let mut building = None;
    if unit_type == &"BuildingTetherPayloadUnit" {
      building = Some(read_int(buf));
    }

    let mut base_rotation = None;
    if unit_type == &"MechUnit" {
      base_rotation = Some(read_float(buf));
    }

    let controller = read_controller(buf);
    let elevation = read_float(buf); // TODO check if 'elv' really is elevation
    let flag = read_double(buf);
    let health = read_float(buf);
    let shooting = read_byte(buf) != 0;

    let mut lifetime = None;
    if unit_type == &"TimedKillUnit" {
      lifetime = Some(read_float(buf));
    }

    let mining_position = read_tile(buf);
    let mounts = read_mounts(buf);

    let mut payloads = None;
    if unit_type == &"PayloadUnit" || unit_type == &"BuildingTetherPayloadUnit" {
      payloads = Some(read_payloads(buf));
    }

    let plans = read_plans(buf);
    let rotation = read_float(buf);
    let shield = read_float(buf);
    let spawned_by_core = read_byte(buf) != 0; // TODO check if 'spbycore' really is spawned_by_core
    let items = read_items(buf);
    let statuses = read_statuses(buf);
    let team = read_byte(buf);

    let mut time = None;
    if unit_type == &"TimedKillUnit" {
      time = Some(read_float(buf));
    }

    let unit_type = read_short(buf); // TODO check if this is what 'utype' actually is
    let upgrade_building = read_byte(buf); // TODO check what 'updbuilding' actually is
    let velocity = read_vec2(buf);
    let x = read_float(buf);
    let y = read_float(buf);

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
      lifetime: read_float(buf),
      tile: read_tile(buf),
      time: read_float(buf),
      x: read_float(buf),
      y: read_float(buf),
    };
  } else if unit_type == &"Puddle" {
    return FullUnit::Puddle {
      revision,
      amount: read_float(buf),
      liquid: read_short(buf),
      tile: read_tile(buf),
      x: read_float(buf),
      y: read_float(buf),
    };
  } else if unit_type == &"Player" {
    return FullUnit::Player {
      revision,
      admin: read_byte(buf) != 0,
      boosting: read_byte(buf) != 0,
      color: read_int(buf),
      mouse_x: read_float(buf),
      mouse_y: read_float(buf),
      name: read_prefixed_string(buf).unwrap(),
      shooting: read_byte(buf) != 0,
      team: read_byte(buf),
      typing: read_byte(buf) != 0,
      unit: read_unit(buf),
      x: read_float(buf),
      y: read_float(buf),
    };
  } else if unit_type == &"WeatherState" {
    return FullUnit::WeatherState {
      revision,
      effect: read_float(buf),
      intensity: read_float(buf),
      life: read_float(buf),
      opacity: read_float(buf),
      weather: read_short(buf),
      wind_x: read_float(buf),
      wind_y: read_float(buf),
    };
  } else if unit_type == &"WorldLabel" {
    return FullUnit::WorldLabel {
      revision,
      flags: read_byte(buf),
      fonts: read_float(buf),
      str: read_prefixed_string(buf).unwrap(),
      x: read_float(buf),
      y: read_float(buf),
    };
  }

  FullUnit::Unknown
}