use super::{cbk::CBK, common::Readable};

pub struct VQM {
    pub width: u32,
    pub height: u32,
    pub cbk_filename: String,
    pub pattern_references: Vec<u16>,
}

impl Readable for VQM {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: super::common::BinaryReader,
    {
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let cbk_filename = reader.read_fixed(16)?;
        let pattern_references = reader.rest_bytes_u16()?;
        Ok(Self {
            width,
            height,
            cbk_filename,
            pattern_references,
        })
    }
}

impl VQM {
    pub fn to_rgba_pixels(&self, cbk: &CBK, lut: &[u32; 256]) -> Vec<u32> {
        let mut rgba_texture: Vec<u32> = vec![0; (self.width * self.height) as usize];

        let mut x = 0;
        let mut y = 0;

        for block in &self.pattern_references {
            let from_clut = (block & 0x8000) == 0x8000;
            let cbk_ref = (block & 0x7fff) as usize;

            for ys in 0..4 {
                let yys = y + ys;
                if yys >= self.height {
                    break;
                }

                for xs in 0..4 {
                    let xxs = x + xs;
                    if xxs >= self.width {
                        break;
                    }

                    let dest_index = (yys * self.width + xxs) as usize;
                    let lut_ref = if from_clut {
                        (block & 0xff) as u8
                    } else {
                        cbk.patterns[cbk_ref][(ys * 4 + xs) as usize]
                    };

                    if lut_ref == 0xff {
                        rgba_texture[dest_index] = 0;
                    } else {
                        rgba_texture[dest_index] = (255 << 24) | lut[lut_ref as usize];
                    }
                }
            }

            x += 4;
            if x >= self.width {
                x = 0;
                y += 4;
            }
        }

        rgba_texture
    }
}
