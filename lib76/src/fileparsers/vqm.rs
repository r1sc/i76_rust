use super::{
    act::ACT,
    binary_reader::{BinaryReader, Readable},
    cbk::CBK,
};

pub struct VQM {
    pub width: u32,
    pub height: u32,
    pub cbk_filename: String,
    pub pattern_references: Vec<u16>,
}

impl Readable for VQM {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
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
    pub fn to_rgba_pixels(&self, cbk: &CBK, act: &ACT, upside_down: bool) -> Vec<u32> {
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

                    let dest_index = if upside_down {
                        ((self.height - 1 - yys) * self.width + xxs) as usize
                    } else {
                        (yys * self.width + xxs) as usize
                    };

                    let lut_ref = if from_clut {
                        (block & 0xff) as u8
                    } else {
                        cbk.patterns[cbk_ref][(ys * 4 + xs) as usize]
                    };

                    if lut_ref == 0xff {
                        rgba_texture[dest_index] = 0;
                    } else {
                        let rgb = act.entries[lut_ref as usize];
                        rgba_texture[dest_index] = (255 << 24)
                            | ((rgb.0 as u32) << 16)
                            | ((rgb.1 as u32) << 8)
                            | (rgb.2 as u32);
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
