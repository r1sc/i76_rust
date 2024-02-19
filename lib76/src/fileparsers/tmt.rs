use super::binary_reader::{BinaryReader, Readable};

pub struct TMT {
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub mode: u32,
    pub num_parts_or_num_groups: u32,
    pub num_parts_if_mode_2: u32,
    pub unk6: u32,
    pub unk7: u32,
    pub f_unk1: f32,
    pub unk8: u32,
    pub unk9: u32,
    pub unk10: u32,
    pub unk11: u32,
    pub unk12: u32,
    pub filenames: Vec<Vec<String>>, // num groups * num parts * 8
    pub num_groups: u32,
    pub num_parts: u32,
}

impl Readable for TMT {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let unk1 = reader.read_u32()?;
        let unk2 = reader.read_u32()?;
        let unk3 = reader.read_u32()?;
        let unk4 = reader.read_u32()?;
        let unk5 = reader.read_u32()?;
        let mode = reader.read_u32()?;
        let num_parts_or_num_groups = reader.read_u32()?;
        let num_parts_if_mode_2 = reader.read_u32()?;
        let unk6 = reader.read_u32()?;
        let unk7 = reader.read_u32()?;
        let f_unk1 = reader.read_f32()?;
        let unk8 = reader.read_u32()?;
        let unk9 = reader.read_u32()?;
        let unk10 = reader.read_u32()?;
        let unk11 = reader.read_u32()?;
        let unk12 = reader.read_u32()?;

        let (num_groups, num_parts) = match mode {
            1 => (1, num_parts_or_num_groups),
            2 => (num_parts_or_num_groups, num_parts_if_mode_2),
            _ => {
                panic!("Unknown TMT mode {}", mode)
            }
        };

        let filenames = (0..num_groups)
            .map(|_| {
                (0..num_parts)
                    .map(|_| reader.read_fixed(8))
                    .collect::<Result<Vec<String>, std::io::Error>>()
            })
            .collect::<Result<Vec<Vec<String>>, std::io::Error>>()?;

        Ok(Self {
            unk1,
            unk2,
            unk3,
            unk4,
            unk5,
            mode,
            num_parts_or_num_groups,
            num_parts_if_mode_2,
            unk6,
            unk7,
            f_unk1,
            unk8,
            unk9,
            unk10,
            unk11,
            unk12,
            filenames,
            num_groups,
            num_parts,
        })
    }
}
