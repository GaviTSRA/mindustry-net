use crate::arc_types::Point2;
use crate::type_io::{
    Reader, Tile, read_command, read_object_boxed, read_prefixed_string, read_string,
    read_vec2_nullable,
};
use crate::unit_io::{Plan, read_payload, read_plans};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Deserialize;
use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MassDriverState {
    Idle = 0,
    Accepting = 1,
    Shooting = 2,
}

#[derive(Debug, Clone)]
pub enum SpecificBlockData {
    // TODO GenericCrafter
    // TODO Separator
    // TODO HeatProducer
    // TODO HeatCrafter
    // TODO AttributeCrafter
    Door {
        open: bool,
    },
    ShieldWall {
        shield: f32,
    },
    MendProjector {
        heat: f32,
        phase_heat: f32,
    },
    OverdriveProjector {
        heat: f32,
        phase_heat: f32,
    },
    ForceProjector {
        broken: bool,
        buildup: f32,
        radius_scale: f32,
        warmup: f32,
        phase_heat: f32,
    },
    Radar {
        progress: f32,
    },
    BuildTurret {
        rotation: f32,
        plans: Vec<Plan>,
    },
    BaseShield {
        smooth_radius: f32,
        broken: bool,
    },
    // TODO Conveyor
    // TODO StackConveyor
    // TODO Junction
    // TODO Bridges (split up?)
    // TODO Sorter
    // TODO OverflowGate
    MassDriver {
        link: u32,
        rotation: f32,
        state: MassDriverState,
    },
    // TODO Duct
    // TODO DuctRouter
    // TODO DirectionalUnloader
    // TODO UnitCargoLoader
    // TODO UnitCargoUnloadPoint
    // TODO reactors (split up?)
    HeaterGenerator {
        heat: f32,
    },
    // TODO drills (split up?)
    Unloader {
        item_id: i16,
    },
    // TODO ItemTurret
    // TODO TractorBeamTurret
    // TODO PointDefenseTurret
    // TODO ContinuousTurret
    // TODO ContinuousLiquidTurret
    RepairTurret {
        rotation: f32,
    },
    // TODO UnitFactory
    // TODO Reconstructor
    // TODO UnitAssembler
    // TODO PayloadConveyor
    // TODO PayloadRouter
    // TODO PayloadMassDriver
    // TODO PayloadDeconstructor
    // TODO Constructor
    // TODO PayloadLoader
    ItemSource {
        item_id: i16,
    },
    LiquidSource {
        liquid_id: i16,
    },
    // TODO PayloadSource
    LightBlock {
        color: u32,
    },
    // TODO LaunchPad
    Accelerator {
        progress: f32,
    },
    Message {
        message: Option<String>,
    },
    Switch {
        enabled: bool,
    },
    // TODO ConsumeGenerator
    // TODO ThermalGenerator
    // TODO SolarGenerator
    // TODO StackRouter
    LiquidTurret {
        reload_counter: f32,
        rotation: f32,
    },
    PowerTurret {
        reload_counter: f32,
        rotation: f32,
    },
    LaserTurret {
        reload_counter: f32,
        rotation: f32,
    },
    // TODO UnitAssemblerModule
    Memory {
        memory: Vec<f64>,
    },
    // TODO LogicDisplay
    // TODO LogicBlock
    Canvas {
        data: Vec<u8>,
    },
    // TODO Build
}

fn read_specific_block_data(
    reader: &mut Reader,
    block_name: String,
    block_type: String,
    version: u8,
    content_map: &HashMap<String, Vec<String>>,
) -> Option<SpecificBlockData> {
    if block_type == "GenericCrafter"
        || block_type == "Separator"
        || block_type == "HeatProducer"
        || block_type == "HeatCrafter"
        || block_type == "AttributeCrafter"
    {
        let progress = reader.float();
        let warmup = reader.float();

        if block_name == "cultivator" {
            reader.float();
        }

        let heat;
        if block_type == "HeatProducer" {
            heat = reader.float();
        }

        let seed;
        if block_type == "Separator" || version == 1 {
            seed = reader.int();
        }

        //let result = {
        //  progress,
        //  warmup,
        //  seed,
        //  heat
        //}
        //return result
    } else if block_type == "Door" || block_type == "AutoDoor" {
        let open = reader.bool();
        return Some(SpecificBlockData::Door { open });
    } else if block_type == "ShieldWall" {
        let shield = reader.float();
        return Some(SpecificBlockData::ShieldWall { shield });
    } else if block_type == "MendProjector" {
        let heat = reader.float();
        let phase_heat = reader.float();
        return Some(SpecificBlockData::MendProjector { heat, phase_heat });
    } else if block_type == "OverdriveProjector" {
        let heat = reader.float();
        let phase_heat = reader.float();
        return Some(SpecificBlockData::OverdriveProjector { heat, phase_heat });
    } else if block_type == "ForceProjector" {
        let broken = reader.bool();
        let buildup = reader.float();
        let radius_scale = reader.float();
        let warmup = reader.float();
        let phase_heat = reader.float();
        return Some(SpecificBlockData::ForceProjector {
            broken,
            buildup,
            radius_scale,
            warmup,
            phase_heat,
        });
    } else if block_type == "Radar" {
        let progress = reader.float();
        return Some(SpecificBlockData::Radar { progress });
    } else if block_type == "BuildTurret" {
        let rotation = reader.float();
        let plans = read_plans(reader);
        return Some(SpecificBlockData::BuildTurret { rotation, plans });
    } else if block_type == "BaseShield" {
        let smooth_radius = reader.float();
        let broken = reader.bool();
        return Some(SpecificBlockData::BaseShield {
            smooth_radius,
            broken,
        });
    } else if block_type == "Conveyor" || block_type == "ArmoredConveyor" {
        let amount = reader.int();
        //let map = []
        for i in 0..amount {
            let id;
            let x;
            let y;
            if version == 0 {
                let val = reader.int();
                id = (val >> 24) & 0xff;
                x = ((val >> 16) & 0xff) / 127;
                y = (((val >> 8) & 0xff) + 128) / 255;
            } else {
                let id = reader.short();
                x = (reader.byte() / 127) as u32;
                y = ((reader.byte() + 128) / 255) as u32
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
        let link = reader.int();
        let cooldown = reader.float();
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
            /*indexes[i] =*/
            reader.byte();
            let length = reader.byte();
            for j in 0..length {
                let value = reader.long();
                //buffers[i][j] = value
            }
        }
        //let result = {
        //  buffers,
        //  indexes
        //}
        //return result
    } else if block_type == "BufferedItemBridge"
        || block_type == "ItemBridge"
        || block_type == "LiquidBridge"
    {
        let link = reader.int();
        let warmup = reader.float();
        let links = reader.byte();
        //let incoming = [];
        let moved;
        for i in 0..links {
            /*incoming.push(*/
            reader.int()/*)*/;
        }
        if version >= 1 {
            moved = reader.byte();
        }
        let index;
        let length;
        //let buffer;
        if block_type == "BufferedItemBridge" {
            index = reader.byte();
            length = reader.byte();
            //buffer = [];
            for i in 0..length {
                let l = reader.long();
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
        let sortitem = reader.short();
        //let buffers = []
        //let indexes = []
        if version == 1 {
            for i in 0..4 {
                //buffers[i] = []
                //indexes[i] = reader.byte();
                let length = reader.byte();
                for j in 0..length {
                    let value = reader.long();
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
                //indexes[i] = reader.byte();
                let length = reader.byte();
                for j in 0..length {
                    let value = reader.long();
                    //buffers[i][j] = value
                }
            }
        } else if version == 3 {
            reader.bytes(4);
        }
        //let result = {
        //  buffers
        //}
        //return result
    } else if block_type == "MassDriver" {
        let link = reader.int();
        let rotation = reader.float();
        let state = reader.byte();
        return Some(SpecificBlockData::MassDriver {
            link,
            rotation,
            state: MassDriverState::try_from(state).unwrap(),
        });
    } else if block_type == "Duct" {
        let recDir;
        if version >= 1 {
            recDir = reader.byte();
        }
        //let result = {
        //  recDir
        //}
        //return result
    } else if block_type == "DuctRouter" {
        let sitem;
        if version >= 1 {
            sitem = reader.short();
        }
        //let result = {
        //  sitem
        //}
        //return result
    } else if block_type == "DirectionalUnloader" {
        let id = reader.short();
        let off = reader.short();
        //let result = {
        //  id,
        //  off
        //}
        //return result
    } else if block_type == "UnitCargoLoader" {
        let unitid = reader.int();
        //let result = {
        //  unitid
        //}
        //return result
    } else if block_type == "UnitCargoUnloadPoint" {
        let item = reader.short();
        let stale = reader.byte();
        //let result = {
        //  item,
        //  stale
        //}
        //return result
    } else if block_type == "NuclearReactor"
        || block_type == "ImpactReactor"
        || block_type == "VariableReactor"
    {
        let peff = reader.float();
        let gentime;
        if version >= 1 {
            gentime = reader.float();
        }
        let heat;
        let warmup;
        let instability;
        if block_type == "NuclearReactor" || block_type == "VariableReactor" {
            heat = reader.float();
        }
        if block_type == "VariableReactor" {
            instability = reader.float();
        }
        if block_type == "ImpactReactor" || block_type == "VariableReactor" {
            warmup = reader.float();
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
        let heat = reader.float();
        return Some(SpecificBlockData::HeaterGenerator { heat });
    } else if block_type == "Drill" || block_type == "BeamDrill" || block_type == "BurstDrill" {
        let progress;
        let warmup;
        let time;
        if version >= 1 {
            if block_type == "Drill" || block_type == "BurstDrill" {
                progress = reader.float();
            } else {
                time = reader.float();
            }
            warmup = reader.float();
        }
        //let result = {
        //  progress,
        //  warmup,
        //  time
        //}
        //return result
    } else if block_type == "Unloader" {
        let item_id = if version == 1 {
            reader.short()
        } else {
            reader.byte() as i16
        };
        return Some(SpecificBlockData::Unloader { item_id });
    } else if block_type == "ItemTurret" {
        let reloadc = reader.float();
        let rotation = reader.float();
        //let ammo = []
        let amount = reader.byte();
        for i in 0..amount {
            let item = reader.short();
            let a = reader.short();
            //ammo[i] = [item, a]
        }
        //let result = {
        //  reloadc,
        //  rotation,
        //  ammo
        //}
        //return result
    } else if block_type == "TractorBeamTurret" || block_type == "PointDefenseTurret" {
        let rotation = reader.float();
        //let result = {
        //  rotation
        //}
        //return result
    } else if block_type == "ContinuousTurret" || block_type == "ContinuousLiquidTurret" {
        let reloadc;
        let rotation;
        if version >= 1 {
            reloadc = reader.float();
            rotation = reader.float();
        }
        let ll;
        if version >= 3 {
            ll = reader.float();
        }
        //let result = {
        //  ll
        //}
        //return result
    } else if block_type == "RepairTurret" {
        let rotation = reader.float();
        return Some(SpecificBlockData::RepairTurret { rotation });
    } else if block_type == "UnitFactory" || block_type == "Reconstructor" {
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let progress;
        if block_type == "UnitFactory" {
            progress = reader.float();
        } else if version >= 1 {
            progress = reader.float();
        }
        let currentplan;
        if block_type == "UnitFactory" {
            currentplan = reader.short();
        }
        let commandpos;
        let command;
        if version >= 2 {
            commandpos = read_vec2_nullable(reader);
        }
        if version >= 3 {
            command = read_command(reader);
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
    } else if block_type == "UnitAssembler" {
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let progress = reader.float();
        let count = reader.byte();
        //let units = [];
        for i in 0..count {
            let unit = reader.int();
            //units.push(unit)
        }
        let pay = read_payload_seq(reader);
        let commandpos;
        if version >= 2 {
            commandpos = read_vec2_nullable(reader);
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
        let progress = reader.float();
        let itemrotation = reader.float();
        let item = read_payload(reader, content_map);
        let sort;
        let recdir;
        if block_type == "PayloadRouter" {
            let ctype = reader.byte();
            sort = reader.short();
            recdir = reader.byte();
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
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let link = reader.int();
        let rotation = reader.float();
        let state = reader.byte();
        let reloadc = reader.float();
        let charge = reader.float();
        let loaded = reader.byte();
        let charging = reader.byte();
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
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let progress = reader.float();
        let accums = reader.short();
        //let accum = [];
        for i in 0..accums {
            /*accum[i] =*/
            reader.float();
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
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let progress = reader.float();
        let rec = reader.short();
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
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let exporting = reader.byte();
        //let result = {
        //  px,
        //  py,
        //  prot,
        //  payload,
        //  exporting
        //}
        //return result
    } else if block_type == "ItemSource" {
        let item_id = reader.short();
        return Some(SpecificBlockData::ItemSource { item_id });
    } else if block_type == "LiquidSource" {
        let liquid_id = reader.short();
        return Some(SpecificBlockData::LiquidSource { liquid_id });
    } else if block_type == "PayloadSource" {
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        let unit = reader.short();
        let block = reader.short();
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
        let color = reader.int();
        return Some(SpecificBlockData::LightBlock { color });
    } else if block_type == "LaunchPad" {
        let lc = reader.float();
        //let result = {
        //  lc
        //}
        //return result
    } else if block_type == "Accelerator" {
        let progress = reader.float();
        return Some(SpecificBlockData::Accelerator { progress });
    } else if block_type == "MessageBlock" {
        let message = read_string(reader);
        return Some(SpecificBlockData::Message { message });
    } else if block_type == "SwitchBlock" {
        let enabled = reader.bool();
        return Some(SpecificBlockData::Switch { enabled });
    } else if block_type == "ConsumeGenerator"
        || block_type == "ThermalGenerator"
        || block_type == "SolarGenerator"
    {
        let pe = reader.float();
        let gentime = reader.float();
        //let result = {
        //  pe,
        //  gentime
        //}
        //return result
    } else if block_type == "StackRouter" {
        let sortitem = reader.short();
        //let result = {
        //  sortitem
        //}
        //return result
    } else if block_type == "LiquidTurret" {
        if version >= 1 {
            let reload_counter = reader.float();
            let rotation = reader.float();
            return Some(SpecificBlockData::LiquidTurret {
                reload_counter,
                rotation,
            });
        }
    } else if block_type == "PowerTurret" {
        if version >= 1 {
            let reload_counter = reader.float();
            let rotation = reader.float();
            return Some(SpecificBlockData::PowerTurret {
                reload_counter,
                rotation,
            });
        }
    } else if block_type == "LaserTurret" {
        if version >= 1 {
            let reload_counter = reader.float();
            let rotation = reader.float();
            return Some(SpecificBlockData::LaserTurret {
                reload_counter,
                rotation,
            });
        }
    } else if block_type == "UnitAssemblerModule" {
        let px = reader.float();
        let py = reader.float();
        let prot = reader.float();
        let payload = read_payload(reader, content_map);
        //let result = {
        //  px,
        //  py,
        //  prot,
        //  payload
        //}
        //return result
    } else if block_type == "MemoryBlock" {
        let amount = reader.int();
        let mut memory = vec![];

        for _ in 0..amount {
            let value = reader.double();
            memory.push(value)
        }

        return Some(SpecificBlockData::Memory { memory });
    } else if block_type == "LogicDisplay" {
        if version >= 1 {
            let has_transform = reader.bool();
            //let map = [];
            if has_transform {
                for i in 0..9 {
                    let val = reader.float();
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
            let compl = reader.int();
            //byte[] bytes = new byte[compl];
            reader.bytes(compl as usize);
            //readCompressed(bytes, false);
        } else {
            let code = read_string(reader);
            //links.clear();
            let total = reader.short();
            for _ in 0..total {
                reader.int();
            }
        }

        let varcount = reader.int();

        //variables need to be temporarily stored in an array until they can be used
        //String[] names = new String[varcount];
        //Object[] values = new Object[varcount];

        for i in 0..varcount {
            let name = read_string(reader);
            /*Object value =*/
            read_object_boxed(reader, true);

            //names[i] = name;
            //values[i] = value;
        }

        let memory = reader.int();
        //skip memory, it isn't used anymore
        reader.bytes((memory * 8) as usize);

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
            let tag = read_prefixed_string(reader);
            let iconTag = reader.short();
        }
    } else if block_type == "CanvasBlock" {
        let length = reader.int();
        let bytes = reader.bytes(length as usize);

        return Some(SpecificBlockData::Canvas { data: bytes });
    } else if block_type.starts_with("Build") {
        let progress = reader.float();
        let pid = reader.short();
        let rid = reader.short();
        let acsize = reader.byte();
        //let acc = []
        //let totalacc = []
        //let itemsLeft = []
        if acsize != 255 {
            for i in 0..acsize {
                /*acc[i] =*/
                reader.float();
                /*totalacc[i] =*/
                reader.float();
                if version >= 1 {
                    /*itemsLeft[i] =*/
                    reader.int();
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
        return None;
    }

    // println!("Unknown block type: {block_type}");
    None
}

fn read_payload_seq(reader: &mut Reader) {
    let amount = reader.short();
    //let ent = []
    for i in 0..(-1 * amount as i16) {
        let payload_type = reader.byte();
        let entr = reader.short();
        let count = reader.int();
    }
    //return ent
}

#[derive(Debug, Clone)]
pub struct BaseBlockData {
    pub health: f32,
    pub rotation: u8,
    pub version: u8,
    pub legacy: bool,
    pub on: Option<u8>,
    pub team: u8,
    pub module_bitmask: u8,
    pub items: Option<HashMap<i16, u32>>,
    pub liquids: Option<HashMap<i16, u32>>,
    pub power: Option<BlockPowerData>,
}
fn read_base_block_data(reader: &mut Reader, id: String) -> BaseBlockData {
    let block_params = load_block_params();

    let health = reader.float();

    let rotation_byte = reader.byte();
    let rotation = rotation_byte & 0b01111111;

    let team = reader.byte();
    let mut version = 0;

    let mut legacy = true;
    let mut on = None;

    let mut module_bitmask = 0;
    if block_params.contains_key(&id) {
        module_bitmask = get_module_bitmask(id, block_params)
    }

    if (rotation_byte & 0b10000000) != 0 {
        version = reader.byte();
        if version >= 1 {
            on = Some(reader.byte());
        }
        if version >= 2 {
            module_bitmask = reader.byte();
        }
        legacy = false;
    }

    let items = if (module_bitmask & 1) != 0 {
        Some(read_block_items(reader, legacy))
    } else {
        None
    };

    let power = if (module_bitmask & 2) != 0 {
        Some(read_block_power(reader))
    } else {
        None
    };

    let liquids = if (module_bitmask & 4) != 0 {
        Some(read_block_liquids(reader, legacy))
    } else {
        None
    };

    if version <= 2 {
        reader.byte();
    }

    let eff;
    let opteff;
    if version >= 3 {
        eff = reader.byte();
        opteff = reader.byte();
    }

    BaseBlockData {
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

fn read_block_items(reader: &mut Reader, legacy: bool) -> HashMap<i16, u32> {
    let count = if legacy {
        reader.byte() as i16
    } else {
        reader.short()
    };

    let mut items = HashMap::new();
    for _ in 0..count {
        let item_id = if legacy {
            reader.byte() as i16
        } else {
            reader.short()
        };
        let item_amount = reader.int();
        items.insert(item_id, item_amount);
    }
    items
}

fn read_block_liquids(reader: &mut Reader, legacy: bool) -> HashMap<i16, u32> {
    let count = if legacy {
        reader.byte() as i16
    } else {
        reader.short()
    };

    let mut liquids = HashMap::new();
    for _ in 0..count {
        let liquid_id = if legacy {
            reader.byte() as i16
        } else {
            reader.short()
        };
        let liquid_amount = reader.int();
        liquids.insert(liquid_id, liquid_amount);
    }
    liquids
}

#[derive(Debug, Clone)]
pub struct BlockPowerData {
    pub links: Vec<Tile>,
    pub status: f32,
}
fn read_block_power(reader: &mut Reader) -> BlockPowerData {
    let amount = reader.short();
    let mut links = vec![];

    for _ in 0..amount {
        let link = reader.int();
        let data = Point2::unpack(link);
        let tile = Tile {
            x: data.x,
            y: data.y,
        };
        links.push(tile)
    }

    let status = reader.float();

    BlockPowerData { links, status }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub base: BaseBlockData,
    pub specific: Option<SpecificBlockData>,
}

pub fn read_block(
    reader: &mut Reader,
    id: String,
    block_type: String,
    version: u8,
    content_map: &HashMap<String, Vec<String>>,
) -> Block {
    let base = read_base_block_data(reader, id.clone());
    let specific = read_specific_block_data(reader, id, block_type, version, content_map);
    Block { base, specific }
}
