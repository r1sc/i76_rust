use super::binary_reader::Readable;

pub struct GPW {
    pub unks: Vec<i32>,
    pub wav_data: Vec<u8>,
}

impl Readable for GPW {
    fn consume(reader: &mut super::binary_reader::BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        assert_eq!(reader.read_fixed(4)?, "GAS0");
        let unks: Vec<_> = (0..6)
            .map(|_| reader.read_i32())
            .collect::<Result<_, _>>()?;
        let wav_data = reader.rest()?;

        Ok(Self { unks, wav_data })
    }
}
