use super::{binary_reader::Readable, common::ColorRGB};

pub struct ACT {
    pub entries: Vec<ColorRGB>,
}
impl Readable for ACT {
    fn consume(reader: &mut super::binary_reader::BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let entries = (0..256)
            .map(|_| ColorRGB::consume(reader))
            .collect::<Result<Vec<ColorRGB>, std::io::Error>>()?;
        Ok(Self { entries })
    }
}
