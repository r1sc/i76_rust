use super::binary_reader::{BinaryReader, Readable};

pub struct VCF {
    pub vcfc: VCFC,
    pub wepns: Vec<WEPN>,
}

impl Readable for VCF {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut vcfc: Option<VCFC> = None;
        let mut wepns: Vec<WEPN> = vec![];

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "VCFC" => vcfc = Some(VCFC::consume(reader)?),
                "WEPN" => {
                    wepns.push(WEPN::consume(reader)?);
                }
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(Self {
            vcfc: vcfc.expect("Expected VCFC"),
            wepns,
        })
    }
}

pub struct VCFC {
    pub variant_name: String, // 16
    pub vdf_filename: String, // 13
    pub vtf_filename: String, // 13
    pub engine_type: u32,
    pub suspension_type: u32,
    pub brakes_type: u32,
    pub wdf_front_filename: String, // 13
    pub wdf_mid_filename: String,   // 13
    pub wdf_back_filename: String,  // 13
    pub armor_front: u32,
    pub armor_left: u32,
    pub armor_right: u32,
    pub armor_rear: u32,
    pub chassis_front: u32,
    pub chassis_left: u32,
    pub chassis_right: u32,
    pub chassis_rear: u32,
    pub armor_or_chassis_left_to_add: u32,
}
impl Readable for VCFC {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let variant_name = reader.read_fixed(16)?;
        let vdf_filename = reader.read_fixed(13)?;
        let vtf_filename = reader.read_fixed(13)?;
        let engine_type = reader.read_u32()?;
        let suspension_type = reader.read_u32()?;
        let brakes_type = reader.read_u32()?;
        let wdf_front_filename = reader.read_fixed(13)?;
        let wdf_mid_filename = reader.read_fixed(13)?;
        let wdf_back_filename = reader.read_fixed(13)?;
        let armor_front = reader.read_u32()?;
        let armor_left = reader.read_u32()?;
        let armor_right = reader.read_u32()?;
        let armor_rear = reader.read_u32()?;
        let chassis_front = reader.read_u32()?;
        let chassis_left = reader.read_u32()?;
        let chassis_right = reader.read_u32()?;
        let chassis_rear = reader.read_u32()?;
        let armor_or_chassis_left_to_add = reader.read_u32()?;
        Ok(Self {
            variant_name,
            vdf_filename,
            vtf_filename,
            engine_type,
            suspension_type,
            brakes_type,
            wdf_front_filename,
            wdf_mid_filename,
            wdf_back_filename,
            armor_front,
            armor_left,
            armor_right,
            armor_rear,
            chassis_front,
            chassis_left,
            chassis_right,
            chassis_rear,
            armor_or_chassis_left_to_add,
        })
    }
}

pub struct WEPN {
    pub mount_point: u32,
    pub gdf_filename: String, // 13
}
impl Readable for WEPN {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mount_point = reader.read_u32()?;
        let gdf_filename = reader.read_fixed(13)?;
        Ok(Self {
            mount_point,
            gdf_filename,
        })
    }
}
