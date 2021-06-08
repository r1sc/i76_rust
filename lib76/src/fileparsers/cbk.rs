use super::binary_reader::{BinaryReader, Readable};

pub struct CBK {
    pub num_entries: u32,
    pub patterns: Vec<Vec<u8>>,
}

impl Readable for CBK {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    {
        let num_entries = reader.read_u32()?;
        let patterns = (0..num_entries)
            .map(|_| reader.bytes(16))
            .collect::<Result<Vec<Vec<_>>, std::io::Error>>()?;

        Ok(Self {
            num_entries,
            patterns,
        })
    }
}
