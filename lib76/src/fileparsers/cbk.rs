use super::common::Readable;

pub struct CBK {
    pub num_entries: u32,
    pub patterns: Vec<Vec<u8>>,
}

impl Readable for CBK {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: super::common::BinaryReader,
    {
        let num_entries = reader.read_u32()?;
        let patterns = (1..num_entries)
            .map(|_| reader.bytes(16))
            .collect::<Result<Vec<Vec<_>>, std::io::Error>>()?;

        Ok(Self {
            num_entries,
            patterns,
        })
    }
}
