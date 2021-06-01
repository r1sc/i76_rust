use super::{
    bwd2::{SCHK, SDFC, SGEO, SOBJ},
    common::{BWD2Tag, Readable},
};

pub struct SDF {
    pub sdfc: SDFC,
    pub sobj: Option<SOBJ>,
    pub sgeo: SGEO,
    pub schks: Vec<SCHK>,
}
impl Readable for SDF {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: super::common::BinaryReader,
    {
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
                    reader.seek((tag.size - 8) as i64)?;
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
