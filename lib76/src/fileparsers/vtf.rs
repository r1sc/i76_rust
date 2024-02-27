use super::binary_reader::{BinaryReader, Readable};

pub struct VTF {
    pub vtfc: VTFC,
}
impl Readable for VTF {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut vtfc: Option<VTFC> = None;

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "VTFC" => vtfc = Some(VTFC::consume(reader)?),
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(VTF {
            vtfc: vtfc.expect("Expected VTFC to be found in VTF"),
        })
    }
}

pub struct VTFC {
    pub vdf_filename: String,       // 13
    pub paint_job_name: String, // 16
    pub parts: Vec<String>,     // 13 * 29
}
impl Readable for VTFC {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let vdf_file = reader.read_fixed(13)?;
        let paint_job_name = reader.read_fixed(16)?;
        let parts = (0..29)
            .map(|_| reader.read_fixed(13))
            .collect::<Result<Vec<String>, std::io::Error>>()?;
        Ok(VTFC {
            vdf_filename: vdf_file,
            paint_job_name,
            parts,
        })
    }
}

pub fn car_texture_name_to_vtf_loc(texture_name: &str) -> u32 {
    let upper = texture_name
        .to_uppercase()
        .replace(".MAP", "")
        .replace(".VQM", "");
    let mut splitted = upper.split_whitespace();
    match (splitted.next(), splitted.next(), splitted.next()) {
        (Some(_), Some(md), Some(rt)) => {
            let major = match md {
                "FT" => 0,
                "MD" => 6,
                "BK" => 12,
                "TP" => 18,
                _ => {
                    panic!("Unknown major car texture part {}", md);
                }
            };
            let minor = match rt {
                "FT" => 0,
                "BK" => 1,
                "RT" => 2,
                "LF" => 3,
                "TP" => 4,
                "UN" => 5,
                _ => {
                    panic!("Unknown minor car texture part {}", rt);
                }
            };
            major + minor
        }
        _ => {
            panic!("Unknown car texture part {}", texture_name);
        }
    }
}
