use either::Either::{Left, Right};

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
        let expected_len = width.checked_mul(height).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "MAP dimensions overflow payload size",
        ))? as usize;
        let available_len = reader.size()? as usize;
        let mut clut_refs = reader.bytes(expected_len.min(available_len))?;

        if clut_refs.len() < expected_len {
            // Some MAP files advertise a full WxH payload but end a few bytes early.
            // Pad the missing refs with transparency instead of failing on EOF.
            clut_refs.resize(expected_len, 0xFF);
        }

        Ok(Self {
            width,
            height,
            clut_refs,
        })
    }
}

impl MAP {
    pub fn to_rgba_pixels(&self, act: &ACT, upside_down: bool) -> Vec<u32> {
        let rows = self.clut_refs.chunks(self.height as usize);

        let iter = if upside_down {
            Left(rows.rev())
        } else {
            Right(rows)
        };

        iter.flat_map(|row| {
            row.iter().map(|clut_ref| {
                let rgb = act.entries[*clut_ref as usize];
                let transparent = *clut_ref == 0xFF || *clut_ref == 1;
                (if transparent { 0 << 24 } else { 255 << 24 })
                    | ((rgb.0 as u32) << 16)
                    | ((rgb.1 as u32) << 8)
                    | (rgb.2 as u32)
            })
        })
        .collect()
    }
}
