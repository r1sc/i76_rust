use super::{
    binary_reader::{BinaryReader, Readable},
    bwd2::{SCHK, SDFC, SGEO, SOBJ},
};

pub struct SDF {
    pub sdfc: SDFC,
    pub sobj: Option<SOBJ>,
    pub sgeo: SGEO,
    pub schks: Vec<SCHK>,
}
impl Readable for SDF {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let mut sdfc: Option<SDFC> = None;
        let mut sobj: Option<SOBJ> = None;
        let mut sgeo: Option<SGEO> = None;
        let mut schks: Vec<SCHK> = vec![];

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "SDFC" => sdfc = Some(SDFC::consume(reader)?),
                "SOBJ" => sobj = Some(SOBJ::consume(reader)?),
                "SGEO" => sgeo = Some(SGEO::consume(reader)?),
                "SCHK" => {
                    let schk = SCHK::consume(reader)?;
                    schks.push(schk);
                }
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(Self {
            sdfc: sdfc.unwrap(),
            sobj,
            sgeo: sgeo.unwrap(),
            schks,
        })
    }
}
