use super::{
    act::ACT,
    binary_reader::{BinaryReader, Readable},
};

pub struct MAP {
    pub width: u32,
    pub height: u32,
    pub clut_refs: Vec<u8>,
}

impl Readable for MAP {
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

impl MAP {
    pub fn to_rgba_pixels(&self, act: &ACT) -> Vec<u32> {
        self.clut_refs
            .iter()
            .map(|clut_ref| {
                let rgb = act.entries[*clut_ref as usize];
                (255 << 24) | ((rgb.0 as u32) << 16) | ((rgb.1 as u32) << 8) | (rgb.2 as u32)
            })
            .collect()
    }
}
