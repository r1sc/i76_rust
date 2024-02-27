use std::rc::Rc;

use crate::{
    fileparsers::{geo::Geo, vcf::VCF, vdf::VDF, vtf::VTF},
    geo_graph::{self, GeoNode},
    virtual_fs::VirtualFS,
};

pub struct CarLod {
    pub damage_state_graphs: Vec<Vec<GeoNode>>,
}

pub struct CarParts {
    pub vcf: VCF,
    pub vdf: VDF,
    pub vtf: VTF,
    pub lods: Vec<CarLod>,
}

impl CarParts {
    pub fn load_car(vcf_filename: &str, vfs: &VirtualFS) -> Self {
        let vcf: VCF = vfs.load(vcf_filename).expect("Failed to load vcf");

        let vtf: VTF = vfs
            .load(&vcf.vcfc.vtf_filename)
            .expect("Failed to load vtf");

        let vdf: VDF = vfs
            .load(&vtf.vtfc.vdf_filename)
            .expect("Failed to load vdf");

        let load_geo = |name: &str| -> Rc<Geo> {
            let filename = format!("{}.geo", name);
            Rc::new(vfs.load::<Geo>(&filename).expect("Failed to load geo"))
        };

        let mut lods = Vec::new();

        for lod_level in &vdf.vgeo.third_person_parts {
            let mut damage_states = Vec::new();

            for damage_state in lod_level {
                let graph =
                    geo_graph::from(damage_state.iter(), load_geo).expect("Failed to build graph");

                damage_states.push(graph);
            }

            lods.push(CarLod {
                damage_state_graphs: damage_states,
            });
        }

        Self {
            vcf,
            vdf,
            vtf,
            lods,
        }
    }
}
