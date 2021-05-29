use nom::{do_parse, named, number::complete::le_u32, take};

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub clut_refs: Vec<u8>,
}
impl super::common::Parsable<Self> for Map {
    named!(
        parse<Map>,
        do_parse!(
            width: le_u32
                >> height: le_u32
                >> clut_refs: take!(width * height)
                >> (Map {
                    width,
                    height,
                    clut_refs: clut_refs.to_owned()
                })
        )
    );
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
