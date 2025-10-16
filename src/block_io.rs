use std::collections::HashMap;
use serde::Deserialize;
use crate::type_io::{read_bool, read_byte, read_command, read_double, read_float, read_int, read_long, read_object_boxed, read_prefixed_string, read_short, read_string, read_vec2_nullable};
use crate::unit_io::{read_payload, read_plans};

// TODO: Everything basically

#[derive(Deserialize)]
#[serde(default)]
struct BlockParam {
  has_items: bool,
  has_power: bool,
  has_liquids: bool,
}
impl Default for BlockParam {
  fn default() -> Self {
    Self {
      has_items: false,
      has_power: false,
      has_liquids: false,
    }
  }
}

fn load_block_params() -> HashMap<String, BlockParam> {
  let data = include_str!("data/block_params.json");
  serde_json::from_str(data).unwrap()
}

fn read_main(data: &mut Vec<u8>, block_type: String, version: u8, content_map: &HashMap<String, Vec<String>>) {
  if block_type == "GenericCrafter" || block_type == "Separator" || block_type == "HeatProducer" || block_type == "HeatCrafter" {
    let progress = read_float(data);
    let warmup = read_float(data);
    
    let heat;
    if block_type == "HeatProducer" {
      heat = read_float(data);
    }
  
    let seed;
    if block_type == "Separator" || version == 1 {
      seed = read_int(data);
    }
  
    //let result = {
    //  progress,
    //  warmup,
    //  seed,
    //  heat
    //}
    //return result
  } else if block_type == "Door" || block_type == "AutoDoor" {
    let open = read_byte(data);
    //let result = {
    //  open
    //}
    //return result
  } else if block_type == "ShieldWall" {
    let shield = read_float(data);
    //let result = {
    // shield
    //
    //return result
  } else if block_type == "MendProjector" || block_type == "OverdriveProjector" {
    let heat = read_float(data);
    let pheat = read_float(data);
    //let result = {
    //  heat,
    //  pheat
    //}
    //return result
  } else if block_type == "ForceProjector" {
    let broken = read_byte(data);
    let buildup = read_float(data);
    let radscl = read_float(data);
    let warmup = read_float(data);
    let pheat = read_float(data);
    //let result = {
    //  broken,
    //  buildup,
    //  radscl,
    //  warmup,
    //  pheat
    //}
    //return result
  } else if block_type == "Radar" {
    let progress = read_float(data);
    //let result = {
    //  progress
    //}
    //return result
  } else if block_type == "BuildTurret" {
    let rotation = read_float(data);
    let plans = read_plans(data);
    //let result = {
    //  rotation,
    //  plans
    //}
    //return result
  } else if block_type == "BaseShield" {
    let sradius = read_float(data);
    let broken = read_byte(data);
    //let result = {
    //  sradius,
    //  broken
    //}
    //return result
  } else if block_type == "Conveyor" || block_type == "ArmoredConveyor"{
    let amount = read_int(data);
    //let map = []
    for i in 0..amount {
      let id;
      let x;
      let y;
      if version == 0 {
        let val = read_int(data);
        id = (val >> 24) & 0xff;
        x = ((val >> 16) & 0xff) / 127;
        y = (((val >> 8) & 0xff) + 128) / 255;
      } else {
        let id = read_short(data);
        x = (read_byte(data) / 127) as u32;
        y = ((read_byte(data) + 128) / 255) as u32
      }
      //let res = {
      //  id,
      //  x,
      //  y
      //}
      //map[i] = res
    }
    //let result = {
    //  map 
    //}
    //return result
  } else if block_type == "StackConveyor" {
    let link = read_int(data);
    let cooldown = read_float(data);
    //let result = {
    //  link,
    //  cooldown
    //}
    //return result
  } else if block_type == "Junction" {
    //let buffers = []
    //let indexes = []
    for i in 0..4 {
      //buffers[i] = []
      /*indexes[i] =*/ read_byte(data);
      let length = read_byte(data);
      for j in 0..length {
        let value = read_long(data);;
        //buffers[i][j] = value
      }
    }
    //let result = {
    //  buffers,
    //  indexes
    //}
    //return result
  } else if block_type == "BufferedItemBridge" || block_type == "ItemBridge" || block_type == "LiquidBridge" {
    let link = read_int(data);
    let warmup = read_float(data);
    let links = read_byte(data);
    //let incoming = [];
    let moved;
    for i in 0..links {
      /*incoming.push(*/read_int(data)/*)*/;
    }
    if version >= 1 {
      moved = read_byte(data);
    }
    let index;
    let length;
    //let buffer;
    if block_type == "BufferedItemBridge" {
      index = read_byte(data);
      length = read_byte(data);
      //buffer = [];
      for i in 0..length {
        let l = read_long(data);;
        //buffer[i] = l;
      }
    }
    //let result = {
    //  link,
    //  warmup,
    //  incoming,
    //  index,
    //  buffer
    //}
    //return result
  } else if block_type == "Sorter" {
    let sortitem = read_short(data);
    //let buffers = []
    //let indexes = []
    if version == 1 {
      for i in 0..4 {
        //buffers[i] = []
        //indexes[i] = read_byte(data);
        let length = read_byte(data);
        for j in 0..length {
          let value = read_long(data);;
          //buffers[i][j] = value
        }
      }
    }
    //let result = {
    //  sortitem,
    //  buffers
    //}
    //return result
  } else if block_type == "OverflowGate" {
    //let buffers = []
    //let indexes = []
    if version == 1 {
      for i in 0..4 {
        //buffers[i] = []
        //indexes[i] = read_byte(data);
        let length = read_byte(data);
        for j in 0..length {
          let value = read_long(data);;
          //buffers[i][j] = value
        }
      }
    } else if version == 3 {
      data.drain(0..4);
    }
    //let result = {
    //  buffers
    //}
    //return result
  } else if block_type == "MassDriver" {
    let link = read_int(data);
    let rotation = read_float(data);
    let state = read_byte(data);
    //let result = {
    //  link,
    //  rotation,
    //  state
    //}
    //return result
  } else if block_type == "Duct" {
    let recDir;
    if version >= 1 {
      recDir = read_byte(data);
    }
    //let result = {
    //  recDir
    //}
    //return result
  } else if block_type == "DuctRouter" {
    let sitem;
    if version >= 1 {
      sitem = read_short(data);
    }
    //let result = {
    //  sitem
    //}
    //return result
  } else if block_type == "DirectionalUnloader"{
    let id = read_short(data);
    let off = read_short(data);
    //let result = {
    //  id,
    //  off
    //}
    //return result
  } else if block_type == "UnitCargoLoader" {
    let unitid = read_int(data);
    //let result = {
    //  unitid
    //}
    //return result
  } else if block_type == "UnitCargoUnloadPoint" {
    let item = read_short(data);
    let stale = read_byte(data);
    //let result = {
    //  item,
    //  stale
    //}
    //return result
  } else if block_type == "NuclearReactor" || block_type == "ImpactReactor" || block_type == "VariableReactor" {
    let peff = read_float(data);
    let gentime;
    if version >= 1 {
      gentime = read_float(data);
    }
    let heat;
    let warmup;
    let instability;
    if block_type == "NuclearReactor" || block_type == "VariableReactor" {
      heat = read_float(data);
    }
    if block_type == "VariableReactor" {
      instability = read_float(data);
    }
    if block_type == "ImpactReactor" || block_type == "VariableReactor"{
      warmup = read_float(data);
    }
    //let result = {
    //  peff,
    //  gentime,
    //  heat,
    //  warmup,
    //  instability
    //}
    //return result
  } else if block_type == "HeaterGenerator" {
    let heat = read_float(data);
    //let result = {
    //  heat
    //}
    //return result
  } else if block_type == "Drill" || block_type == "BeamDrill" || block_type == "BurstDrill" {
    let progress;
    let warmup;
    let time;
    if version >= 1 {
      if block_type == "Drill" || block_type == "BurstDrill" {
        progress = read_float(data);
      } else {
        time = read_float(data);
      }
      warmup = read_float(data);
    }
    //let result = {
    //  progress,
    //  warmup,
    //  time
    //}
    //return result
  } else if block_type == "Unloader" {
    let id;
    if version == 1 {
      id = read_short(data);
    } else {
      /*id =*/ read_byte(data);
    }
    //let result = {
    //  id
    //}
    //return result
  } else if block_type == "ItemTurret" {
    let reloadc = read_float(data);
    let rotation = read_float(data);
    //let ammo = []
    let amount = read_byte(data);
    for i in 0..amount {
      let item = read_short(data);
      let a = read_short(data);
      //ammo[i] = [item, a]
    }
    //let result = {
    //  reloadc,
    //  rotation,
    //  ammo
    //}
    //return result
  } else if block_type == "TractorBeamTurret" || block_type == "PointDefenseTurret" {
    let rotation = read_float(data);
    //let result = {
    //  rotation
    //}
    //return result
  } else if block_type == "ContinuousTurret" || block_type == "ContinuousLiquidTurret" {
    let reloadc;
    let rotation;
    if version >= 1 {
      reloadc = read_float(data);
      rotation = read_float(data);
    }
    let ll;
    if version >= 3 {
      ll = read_float(data);
    }
    //let result = {
    //  ll
    //}
    //return result
  } else if block_type == "UnitFactory" || block_type == "Reconstructor" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);
    let progress;
    if block_type == "UnitFactory" {
      progress = read_float(data);
    } else if version >= 1 {
      progress = read_float(data);
    }
    let currentplan;
    if block_type == "UnitFactory" {
      currentplan = read_short(data);
    }
    let commandpos;
    let command;
    if version >= 2 {
      commandpos = read_vec2_nullable(data);
    }
    if version >= 3 {
      command = read_command(data);
    }
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  progress,
    //  currentplan,
    //  commandpos,
    //  command
    //}
    //return result
  } else if block_type == "RepairTurret" {
    let rotation = read_float(data);
    //let result = {
    //  rotation
    //}
    //return result
  } else if block_type == "UnitAssembler" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);
    let progress = read_float(data);
    let count = read_byte(data);
    //let units = [];
    for i in 0..count {
      let unit = read_int(data);
      //units.push(unit)
    }
    let pay = read_payload_seq(data);
    let commandpos;
    if version >= 2 {
      commandpos = read_vec2_nullable(data);
    }
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  progress,
    //  units,
    //  pay,
    //  commandpos
    //}
    //return result
  } else if block_type == "PayloadConveyor" || block_type == "PayloadRouter" {
    let progress = read_float(data);
    let itemrotation = read_float(data);
    let item = read_payload(data, content_map);
    let sort;
    let recdir;
    if block_type == "PayloadRouter" {
      let ctype = read_byte(data);
      sort = read_short(data);
      recdir = read_byte(data);
    }
    //let result = {
    //  progress,
    //  itemrotation,
    //  item,
    //  sort,
    //  recdir
    //}
    //return result
  } else if block_type == "PayloadMassDriver" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);
    let link = read_int(data);
    let rotation = read_float(data);
    let state = read_byte(data);
    let reloadc = read_float(data);
    let charge = read_float(data);
    let loaded = read_byte(data);
    let charging = read_byte(data);
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  link,
    //  rotation,
    //  state,
    //  reloadc,
    //  charge,
    //  loaded,
    //  charging
    //}
    //return result
  } else if block_type == "PayloadDeconstructor" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);;
    let progress = read_float(data);
    let accums = read_short(data);
    //let accum = [];
    for i in 0..accums {
      /*accum[i] =*/ read_float(data);
    }
    //let decp;
    //[decp, offset] = rpl(buf, offset);
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  progress,
    //  accum,
    //  decp
    //}
    //return result
  } else if block_type == "Constructor" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);;
    let progress = read_float(data);
    let rec = read_short(data);
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  progress,
    //  rec
    //}
    //return result
  } else if block_type == "PayloadLoader" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);
    let exporting = read_byte(data);
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  exporting
    //}
    //return result
  } else if block_type == "ItemSource" {
    let item = read_short(data);
    //let result = {
    //  item
    //}
    //return result
  } else if block_type == "LiquidSource" {
    let id = read_short(data);
    //let result = {
    //  id
    //}
    //return result
  } else if block_type == "PayloadSource" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);
    let unit = read_short(data);
    let block = read_short(data);
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload,
    //  unit,
    //  block
    //}
    //return result
  } else if block_type == "LightBlock" {
    let color = read_int(data);
    //let result = {
    //  color
    //}
    //return result
  } else if block_type == "LaunchPad" {
    let lc = read_float(data);
    //let result = {
    //  lc
    //}
    //return result
  } else if block_type == "Accelerator" {
    let progress = read_float(data);
    //let result = {
    //  progress
    //}
    //return result
  } else if block_type == "MessageBlock" {
    let str = read_string(data);
    //let result = {
    //  str
    //}
    //return result
  } else if block_type == "SwitchBlock" {
    let en = read_byte(data);
    //let result = {
    //  en
    //}
    //return result
  } else if block_type == "ConsumeGenerator" || block_type == "ThermalGenerator" || block_type == "SolarGenerator" {
    let pe = read_float(data);
    let gentime = read_float(data);
    //let result = {
    //  pe,
    //  gentime
    //}
    //return result
  } else if block_type == "StackRouter" {
    let sortitem = read_short(data);
    //let result = {
    //  sortitem
    //}
    //return result
  } else if block_type == "LiquidTurret" || block_type == "PowerTurret" || block_type == "LaserTurret" {
    let reloadc = read_float(data);
    let rotation = read_float(data);
    //let result = {
    //  reloadc,
    //  rotation
    //}
    //return result
  } else if block_type == "UnitAssemblerModule" {
    let px = read_float(data);
    let py = read_float(data);
    let prot = read_float(data);
    let payload = read_payload(data, content_map);
    //let result = {
    //  px,
    //  py,
    //  prot,
    //  payload
    //}
    //return result
  } else if block_type == "MemoryBlock" {
    let amount = read_int(data);
    //let memory = [];
    for i in 0..amount {
      let value = read_double(data);
      //memory[i] = value
    }
    //let result = {
    //  memory
    //}
    //return result
  } else if block_type == "LogicDisplay" {
    if version >= 1 {
      let has_transform = read_bool(data);
      println!("{has_transform} {version}");
      //let map = [];
      if has_transform {
        for i in 0..9 {
          let val = read_float(data);
          //map[i] = val
        }
      }
      //let result = {
      //  map
      //}
      //return result
    }
  } else if block_type == "LogicBlock" {
    if version >= 1 {
      let compl = read_int(data);
      //byte[] bytes = new byte[compl];
      data.drain(..compl as usize);
      //readCompressed(bytes, false);
    } else {
      let code = read_string(data);
      //links.clear();
      let total = read_short(data);
      for _ in 0..total {
        read_int(data);
      }
    }

    let varcount = read_int(data);

    //variables need to be temporarily stored in an array until they can be used
    //String[] names = new String[varcount];
    //Object[] values = new Object[varcount];

    for i in 0..varcount {
      let name = read_string(data);
      /*Object value =*/ read_object_boxed(data, true);

      //names[i] = name;
      //values[i] = value;
    }

    let memory = read_int(data);
    //skip memory, it isn't used anymore
    data.drain(0..(memory * 8) as usize);

    //loadBlock = () -> updateCode(code, false, asm -> {
    //  //load up the variables that were stored
    //  for(int i = 0; i < varcount; i++){
    //    LVar var = asm.getVar(names[i]);
    //    if(var != null && (!var.constant || var.name.equals("@unit"))){
    //      var value = values[i];
    //      if(value instanceof Boxed<?> boxed) value = boxed.unbox();
    //
    //      if(value instanceof Number num){
    //        var.numval = num.doubleValue();
    //        var.isobj = false;
    //      }else{
    //        var.objval = value;
    //        var.isobj = true;
    //      }
    //    }
    //  }
    //});

    //if privileged && version >= 2){
    //  ipt = Mathf.clamp(read.s(), 1, maxInstructionsPerTick);
    //}

    if version >= 3 {
      let tag = read_prefixed_string(data);
      let iconTag = read_short(data);
    }
  } else if block_type == "CanvasBlock" {
    let length = read_int(data);
    let bytes = data.drain(..length as usize).collect::<Vec<u8>>();
    //let result = {
    //  data
    //}
    //return result
  } else if block_type.starts_with("Build") {
    let progress = read_float(data);
    let pid = read_short(data);
    let rid = read_short(data);
    let acsize = read_byte(data);
    //let acc = []
    //let totalacc = []
    //let itemsLeft = []
    if acsize != 255 {
      for i in 0..acsize {
        /*acc[i] =*/ read_float(data);
        /*totalacc[i] =*/ read_float(data);
        if version >= 1 {
          /*itemsLeft[i] =*/ read_int(data);
        }
      }
    }
    //let result = {
    //  progress,
    //  pid,
    //  rid,
    //  acsize,
    //  acc,
    //  totalacc,
    //  itemsLeft
    //}
    //return result
  } else {
    //return null
  }
}
fn read_payload_seq(data: &mut Vec<u8>) {
  let amount = read_short(data);
  //let ent = []
  for i in 0..(-1 * amount as i16) {
    let payload_type = read_byte(data);
    let entr = read_short(data);
    let count = read_int(data);
  }
  //return ent
}

#[derive(Debug)]
struct BlockBaseData {
  health: f32,
  rotation: u8,
  version: u8,
  legacy: bool,
  on: Option<u8>,
  team: u8,
  module_bitmask: u8,
  items: Option<HashMap<u16, u32>>,
  liquids: Option<HashMap<u16, u32>>,
  power: Option<BlockPowerData>,
}
fn read_base_block_data(data: &mut Vec<u8>, id: String) -> BlockBaseData {
  let block_params = load_block_params();

  let health = read_float(data);

  let rotation_byte = read_byte(data);
  let rotation = rotation_byte & 0b01111111;

  let team = read_byte(data);
  let mut version = 0;

  let mut legacy = true;
  let mut on = None;

  let mut module_bitmask = 0;
  if block_params.contains_key(&id) {
    module_bitmask = get_module_bitmask(id, block_params)
  }

  if (rotation_byte & 0b10000000) != 0 {
    version = read_byte(data);
    if version >= 1 {
      on = Some(read_byte(data));
    }
    if version >= 2 {
      module_bitmask = read_byte(data);
    }
    legacy = false;
  }

  let items = if (module_bitmask & 1) != 0 {
    Some(read_block_items(data, legacy))
  } else { None };

  let power= if (module_bitmask & 2) != 0 {
    Some(read_block_power(data))
  } else { None };

  let liquids= if (module_bitmask & 4) != 0 {
    Some(read_block_liquids(data, legacy))
  } else { None };

  if version <= 2 {
    read_byte(data);
  }

  let eff;
  let opteff;
  if version >= 3 {
    eff = read_byte(data);
    opteff = read_byte(data);
  }

  BlockBaseData {
    health,
    rotation,
    team,
    version,
    module_bitmask,
    legacy,
    on,
    items,
    power,
    liquids,
    //eff,  TODO
    //opteff
  }
}

fn get_module_bitmask(id: String, block_parms: HashMap<String, BlockParam>) -> u8 {
  let has_items = block_parms.get(&id).unwrap().has_items;
  let has_power = block_parms.get(&id).unwrap().has_power;
  let has_liquids = block_parms.get(&id).unwrap().has_liquids;

  let a = if has_items { 1 } else { 0 };
  let b = if has_power { 2 } else { 0 };
  let c = if has_liquids { 4 } else { 0 };
  a | b | c | 8
}

fn read_block_items(data: &mut Vec<u8>, legacy: bool) -> HashMap<u16, u32> {
  let count = if legacy {
    read_byte(data) as u16
  } else {
    read_short(data)
  };

  let mut items = HashMap::new();
  for _ in 0..count {
    let item_id = if legacy {
      read_byte(data) as u16
    } else {
      read_short(data)
    };
    let item_amount = read_int(data);
    items.insert(item_id, item_amount);
  }
  items
}

fn read_block_liquids(data: &mut Vec<u8>, legacy: bool) -> HashMap<u16, u32> {
  let count = if legacy {
    read_byte(data) as u16
  } else {
    read_short(data)
  };

  let mut liquids = HashMap::new();
  for _ in 0..count {
    let liquid_id = if legacy {
      read_byte(data) as u16
    } else {
      read_short(data)
    };
    let liquid_amount = read_int(data);
    liquids.insert(liquid_id, liquid_amount);
  }
  liquids
}

#[derive(Debug)]
struct BlockPowerData {
  links: Vec<u32>,
  status: f32,
}
fn read_block_power(data: &mut Vec<u8>) -> BlockPowerData {
  let amount = read_short(data);;
  let mut links = vec![];

  for _ in 0..amount {
    let link  = read_int(data);
    links.push(link)
  }

  let status = read_float(data);;

  BlockPowerData { links, status }
}

pub fn readAll(data: &mut Vec<u8>, id: String, block_type: String, version: u8, content_map:  &HashMap<String, Vec<String>>) {
  let base = read_base_block_data(data, id);

  let main = read_main(data, block_type, version, content_map);
  //return [base, main]
}