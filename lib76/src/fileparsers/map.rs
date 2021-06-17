use super::{act::ACT, binary_reader::{BinaryReader, Readable}, common::ColorRGB};

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub clut_refs: Vec<u8>,
}

impl Readable for Map {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let clut_refs = reader.bytes((width * height) as usize)?;
        Ok(Self {
            width,
            height,
            clut_refs,
        })
    }
}

impl Map {
    pub fn to_rgba_pixels(&self, act: &ACT) -> Vec<u32> {
        self.clut_refs
            .iter()
            .map(|clut_ref| {
                // match act {
                //     Some(a) => {
                        let rgb = act.entries[*clut_ref as usize];
                        (255 << 24) | ((rgb.0 as u32) << 16) | ((rgb.1 as u32) << 8) | (rgb.2 as u32)
                    // },
                    // None => {
                    //     if clut_ref == &0xffu8 {
                    //         0
                    //     } else {
                    //         let rgb = lut[*clut_ref as usize];
                    //         (255 << 24) | rgb
                    //     }
                    // }
                // }
            })
            .collect()
    }
}
