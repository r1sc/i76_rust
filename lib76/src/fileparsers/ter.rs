use super::binary_reader::Readable;

pub struct TER {
    pub entries: Vec<Vec<u16>>,
}

impl Readable for TER {
    fn consume(reader: &mut super::binary_reader::BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let len = reader.len()?;

        let block_size = 128 * 128;
        let entry_size = 2 * block_size;
        let num_entries = len / entry_size;

        let entries = (0..num_entries)
            .map(|_| {
                (0..block_size)
                    .map(|_| reader.read_u16())
                    .collect::<Result<_, std::io::Error>>()
            })
            .collect::<Result<_, std::io::Error>>()?;

        Ok(Self { entries })
    }
}
