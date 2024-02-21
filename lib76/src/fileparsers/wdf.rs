use super::{
    binary_reader::{BinaryReader, Readable},
    bwd2::{WDFC, WGEO},
};

#[derive(Debug)]
pub struct WDF {
    pub wdfc: WDFC,
    pub wgeo: WGEO

}
impl Readable for WDF {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let mut wdfc: Option<WDFC> = None;
        let mut wgeo: Option<WGEO> = None;

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "WDFC" => wdfc = Some(WDFC::consume(reader)?),
                "WGEO" => wgeo = Some(WGEO::consume(reader)?),
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(Self {
            wdfc: wdfc.unwrap(),
            wgeo: wgeo.unwrap(),
        })
    }
}
