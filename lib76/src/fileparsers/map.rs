use super::common::Readable;

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub clut_refs: Vec<u8>,
}

impl Readable for Map {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error> where R : super::common::BinaryReader {
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let clut_refs = reader.bytes((width * height) as usize)?;
        Ok(Self {
            width,
            height,
            clut_refs
        })
    }
}

impl Map {
    pub fn to_rgba_pixels(&self, lut: &[u32; 256]) -> Vec<u32> {
        self.clut_refs
            .iter()
            .map(|clut_ref| {
                let rgb = lut[*clut_ref as usize];
                (255 << 24) | rgb
            })
            .collect()
    }
}
