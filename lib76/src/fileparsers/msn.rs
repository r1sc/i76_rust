use std::{collections::VecDeque, io::Seek};

use glam::Vec3;

use super::{
    binary_reader::{BinaryReader, Readable},
    common::RotationAxis,
};

pub struct MSN {
    pub wrev: WREV,
    pub wrld: WRLD,
    pub rsegs: Vec<RSEG>,
    pub odef_objs: Vec<ODEFObj>,
    pub ldef_objs: Vec<LDEFObj>,
    pub tdef: TDEF,
    pub fsm: FSM,
}

impl Readable for MSN {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let mut wrev: Option<WREV> = None;
        let mut wrld: Option<WRLD> = None;
        let mut rsegs: Vec<RSEG> = vec![];
        let mut odef_objs: Vec<ODEFObj> = vec![];
        let mut ldef_objs: Vec<LDEFObj> = vec![];
        let mut tdef: Option<TDEF> = None;
        let mut fsm: Option<FSM> = None;

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "WDEF" => {
                    while let Ok(tag) = reader.bwd2_tag() {
                        match &tag.name[..] {
                            "WREV" => wrev = Some(WREV::consume(reader)?),
                            "WRLD" => wrld = Some(WRLD::consume(reader)?),
                            "EXIT" => {
                                break;
                            }
                            _ => {
                                reader.seek_relative(tag.size as i64)?;
                            }
                        }
                    }
                }
                "RDEF" => {
                    while let Ok(tag) = reader.bwd2_tag() {
                        match &tag.name[..] {
                            "RSEG" => {
                                let obj = RSEG::consume(reader)?;
                                rsegs.push(obj);
                            }
                            "EXIT" => {
                                break;
                            }
                            _ => {
                                reader.seek_relative(tag.size as i64)?;
                            }
                        }
                    }
                }
                "ODEF" => {
                    while let Ok(tag) = reader.bwd2_tag() {
                        match &tag.name[..] {
                            "OBJ" => {
                                let obj = ODEFObj::consume(reader)?;
                                odef_objs.push(obj);
                            }
                            "EXIT" => {
                                break;
                            }
                            _ => {
                                reader.seek_relative(tag.size as i64)?;
                            }
                        }
                    }
                }
                "LDEF" => {
                    while let Ok(tag) = reader.bwd2_tag() {
                        match &tag.name[..] {
                            "OBJ" => {
                                let obj = LDEFObj::consume(reader)?;
                                ldef_objs.push(obj);
                            }
                            "EXIT" => {
                                break;
                            }
                            _ => {
                                reader.seek_relative(tag.size as i64)?;
                            }
                        }
                    }
                }
                "TDEF" => tdef = Some(TDEF::consume(reader)?),
                "ADEF" => {
                    while let Ok(tag) = reader.bwd2_tag() {
                        match &tag.name[..] {
                            "FSM" => {
                                fsm = Some(FSM::consume(reader)?);
                            }
                            _ => {
                                reader.seek_relative(tag.size as i64)?;
                            }
                        }
                    }
                }
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(Self {
            wrev: wrev.expect("Expected WREV"),
            wrld: wrld.expect("Expected WRLD"),
            rsegs,
            odef_objs,
            ldef_objs,
            tdef: tdef.expect("Expected TDEF"),
            fsm: fsm.expect("Expected FSM"),
        })
    }
}

pub struct SurfaceParams {
    pub friction: f32,
    pub rolling_resistance: f32,
    pub roughness: f32,
    pub visual_quality: u32,
    pub ddr_per_sec: u32, // I think this is damage per sec
}
impl Readable for SurfaceParams {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let friction = reader.read_f32()?;
        let rolling_resistance = reader.read_f32()?;
        let roughness = reader.read_f32()?;
        let visual_quality = reader.read_u32()?;
        let ddr_per_sec = reader.read_u32()?;

        Ok(Self {
            friction,
            rolling_resistance,
            roughness,
            visual_quality,
            ddr_per_sec,
        })
    }
}

pub struct WREV {
    pub revision: u32,
}
impl Readable for WREV {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let revision = reader.read_u32()?;
        Ok(Self { revision })
    }
}

pub struct WRLD {
    pub cd_track: u32,
    pub intro_smk_filename: String,       // 13
    pub outro_smk_filename: String,       // 13
    pub act_filename: String,             // 13
    pub lum_filename: String,             // 13
    pub tbl_filename: String,             // 13
    pub npt_filename: String,             // 13
    pub sky_texture_filename: String,     // 13
    pub scrounge_sdf_filename: String,    // 13
    pub surface_texture_filename: String, // 13
    pub level_map_filename: String,       // 13
    pub time_of_day: u32,
    pub surface_params_dirt: SurfaceParams,
    pub surface_params_light_veg: SurfaceParams,
    pub surface_params_paved_road: SurfaceParams,
    pub surface_params_packed_dirt: SurfaceParams,
    pub surface_params_wash_road: SurfaceParams,
    pub surface_params_rocky: SurfaceParams,
    pub surface_params_unk1: SurfaceParams,
    pub surface_params_unk2: SurfaceParams,
    pub far_clip_dist: u32,
    pub description: String, // 16
}
impl Readable for WRLD {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let cd_track = reader.read_u32()?;
        let intro_smk_filename = reader.read_fixed(13)?;
        let outro_smk_filename = reader.read_fixed(13)?;
        let act_filename = reader.read_fixed(13)?;
        let lum_filename = reader.read_fixed(13)?;
        let tbl_filename = reader.read_fixed(13)?;
        let npt_filename = reader.read_fixed(13)?;
        let sky_texture_filename = reader.read_fixed(13)?;
        let scrounge_sdf_filename = reader.read_fixed(13)?;
        let surface_texture_filename = reader.read_fixed(13)?;
        let level_map_filename = reader.read_fixed(13)?;
        let _horizon_definition_filename = reader.read_fixed(13)?;
        let time_of_day = reader.read_u32()?;
        let surface_params_dirt = SurfaceParams::consume(reader)?;
        let surface_params_light_veg = SurfaceParams::consume(reader)?;
        let surface_params_paved_road = SurfaceParams::consume(reader)?;
        let surface_params_packed_dirt = SurfaceParams::consume(reader)?;
        let surface_params_wash_road = SurfaceParams::consume(reader)?;
        let surface_params_rocky = SurfaceParams::consume(reader)?;
        let surface_params_unk1 = SurfaceParams::consume(reader)?;
        let surface_params_unk2 = SurfaceParams::consume(reader)?;
        let far_clip_dist = reader.read_u32()?;
        let description = reader.read_fixed(16)?;

        Ok(Self {
            cd_track,
            intro_smk_filename,
            outro_smk_filename,
            act_filename,
            lum_filename,
            tbl_filename,
            npt_filename,
            sky_texture_filename,
            scrounge_sdf_filename,
            surface_texture_filename,
            level_map_filename,
            time_of_day,
            surface_params_dirt,
            surface_params_light_veg,
            surface_params_paved_road,
            surface_params_packed_dirt,
            surface_params_wash_road,
            surface_params_rocky,
            surface_params_unk1,
            surface_params_unk2,
            far_clip_dist,
            description,
        })
    }
}

pub struct RSEG {
    pub segment_type: u32,
    pub segment_piece_count: u32,
    pub pieces: Vec<(Vec3, Vec3)>,
}
impl Readable for RSEG {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let segment_type = reader.read_u32()?;
        let segment_piece_count = reader.read_u32()?;
        let pieces = (0..segment_piece_count)
            .map(|_| {
                let left = Vec3::consume(reader)?;
                let right = Vec3::consume(reader)?;
                Ok((left, right))
            })
            .collect::<Result<_, std::io::Error>>()?;
        Ok(Self {
            segment_type,
            segment_piece_count,
            pieces,
        })
    }
}

pub struct ODEFObj {
    pub label: String, // 8
    pub rotation: RotationAxis,
    pub position: Vec3,
    pub unk: Vec<u32>, // * 9
    pub class_id: u32,
    pub flags: u16,
    pub team_id: u16,
}
impl Readable for ODEFObj {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let label = reader.read_fixed(8)?;
        let rotation = RotationAxis::consume(reader)?;
        let position = Vec3::consume(reader)?;
        let unk = (0..9)
            .map(|_| reader.read_u32())
            .collect::<Result<_, std::io::Error>>()?;
        let class_id = reader.read_u32()?;
        let flags = reader.read_u16()?;
        let team_id = reader.read_u16()?;

        Ok(Self {
            label,
            rotation,
            position,
            unk,
            class_id,
            flags,
            team_id,
        })
    }
}

pub struct LDEFObj {
    pub label: String, // 100
    pub num_strings: u32,
    pub string_positions: Vec<Vec3>, // * num_strings
}
impl Readable for LDEFObj {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let label = reader.read_fixed(100)?;
        let num_strings = reader.read_u32()?;
        let string_positions = (0..num_strings)
            .map(|_| Vec3::consume(reader))
            .collect::<Result<_, std::io::Error>>()?;

        Ok(Self {
            label,
            num_strings,
            string_positions,
        })
    }
}

pub struct TREV {
    pub revision: u32,
}
impl Readable for TREV {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let revision = reader.read_u32()?;
        Ok(Self { revision })
    }
}

pub struct ZMAP {
    pub num_active_zones: u8,
    pub zone_references: Vec<u8>, // 80 * 80
}
impl Readable for ZMAP {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let num_active_zones = reader.read_u8()?;
        let zone_references = reader.bytes((80 * 80) as usize)?;
        Ok(Self {
            num_active_zones,
            zone_references,
        })
    }
}

pub struct ZONE {
    pub unk: u8,
    pub ter_filename: String, // 13
}
impl Readable for ZONE {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let unk = reader.read_u8()?;
        let ter_filename = reader.read_fixed(13)?;
        Ok(Self { unk, ter_filename })
    }
}

pub struct TDEF {
    pub trev: TREV,
    pub zmap: ZMAP,
    pub zone: ZONE,
}
impl Readable for TDEF {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut trev: Option<TREV> = None;
        let mut zmap: Option<ZMAP> = None;
        let mut zone: Option<ZONE> = None;

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "TREV" => trev = Some(TREV::consume(reader)?),
                "ZMAP" => zmap = Some(ZMAP::consume(reader)?),
                "ZONE" => zone = Some(ZONE::consume(reader)?),
                "EXIT" => {
                    break;
                }
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(Self {
            trev: trev.expect("Expected TREV"),
            zmap: zmap.expect("Expected ZMAP"),
            zone: zone.expect("Expected ZONE"),
        })
    }
}

pub struct FSM {
    pub action_table: Vec<String>,
    pub entity_table: Vec<FSMEntity>,
    pub sound_clip_table: Vec<String>,
    pub paths: Vec<FSMPath>,
    pub stack_machine_definitions: Vec<FSMStackMachineDefinition>,
    pub raw_instructions: Vec<FSMRawInstruction>,
    pub constants: Vec<i32>,
}

impl Readable for FSM {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let action_table_size = reader.read_u32()? as usize;
        let mut action_table = Vec::with_capacity(action_table_size);
        for _ in 0..action_table_size {
            action_table.push(reader.read_fixed(40)?);
        }

        let num_entities = reader.read_u32()? as usize;
        let mut entity_table = Vec::with_capacity(num_entities);
        for _ in 0..num_entities {
            let label = reader.read_fixed(40)?;
            let raw_label = reader.bytes(8)?;

            let mut label_high: i32 = 0;
            let mut label_builder = String::new();

            for j in 0..8 {
                let mut v = raw_label[j];
                if v > 0x7f {
                    label_high = (label_high << 1) | 0x01;
                } else {
                    label_high = (label_high << 1) & 0xfe;
                }

                v &= 0x7f;
                if v != 0 {
                    label_builder.push(v as char);
                }
            }

            entity_table.push(FSMEntity {
                label,
                value: label_builder,
                id: label_high,
            });
        }

        let num_sound_clips = reader.read_u32()? as usize;
        let mut sound_clip_table = Vec::with_capacity(num_sound_clips);
        for _ in 0..num_sound_clips {
            sound_clip_table.push(reader.read_fixed(40)?);
        }

        let num_paths = reader.read_u32()? as usize;
        let mut paths = Vec::with_capacity(num_paths);
        for _ in 0..num_paths {
            let name = reader.read_fixed(40)?;
            let num_nodes = reader.read_u32()? as usize;
            let mut points = Vec::with_capacity(num_nodes);
            for _ in 0..num_nodes {
                points.push(Vec3::consume(reader)?);
            }

            paths.push(FSMPath { name, points });
        }

        let num_machines = reader.read_u32()? as usize;
        let mut stack_machines = Vec::with_capacity(num_machines);
        for _ in 0..num_machines {
            let next = reader.reader.stream_position()? + 168;

            let start_address = reader.read_u32()?;
            let num_initial_arguments = reader.read_u32()? as usize;
            let mut initial_arguments = Vec::with_capacity(num_initial_arguments);
            for _ in 0..num_initial_arguments {
                initial_arguments.push(reader.read_i32()?);
            }

            stack_machines.push(FSMStackMachineDefinition {
                start_address,
                initial_arguments,
            });

            reader.seek_from_start(next)?;
        }

        let num_constants = reader.read_u32()? as usize;
        let mut constants = Vec::with_capacity(num_constants);
        for _ in 0..num_constants {
            constants.push(reader.read_i32()?);
        }

        let num_raw_instructions = reader.read_u32()? as usize;
        let mut raw_instructions = Vec::with_capacity(num_raw_instructions);
        for _ in 0..num_raw_instructions {
            let opcode = reader.read_u32()?;
            let value = reader.read_i32()?;
            raw_instructions.push(FSMRawInstruction {
                opcode: opcode
                    .try_into()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
                value,
            });
        }

        Ok(Self {
            action_table,
            entity_table,
            paths,
            raw_instructions,
            sound_clip_table,
            stack_machine_definitions: stack_machines,
            constants,
        })
    }
}

pub struct FSMEntity {
    pub label: String,
    pub value: String,
    pub id: i32,
}

pub struct FSMPath {
    pub name: String,
    pub points: Vec<Vec3>,
}

pub struct FSMStackMachineDefinition {
    pub start_address: u32,
    pub initial_arguments: Vec<i32>,
}

#[derive(Debug)]
pub enum FSMOpcode {
    Push,
    ArgPushS,
    ArgPushB,
    Adjust, // Adjust by arg
    Drop,   // Set stack pointer to SP-arg
    Jmp,
    Jz,
    JmpI,
    Rst,
    Action,
    Neg,
}

impl TryFrom<u32> for FSMOpcode {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Push),
            4 => Ok(Self::ArgPushS),
            5 => Ok(Self::ArgPushB),
            6 => Ok(Self::Adjust),
            7 => Ok(Self::Drop),
            8 => Ok(Self::Jmp),
            9 => Ok(Self::Jz),
            10 => Ok(Self::JmpI),
            12 => Ok(Self::Rst),
            13 => Ok(Self::Action),
            14 => Ok(Self::Neg),
            _ => Err(format!("Unknown FSM opcode {}", value)),
        }
    }
}

pub struct FSMRawInstruction {
    pub opcode: FSMOpcode,
    pub value: i32,
}
