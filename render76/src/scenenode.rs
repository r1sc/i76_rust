use std::{cell::RefCell, rc::Rc};

use glam::{vec3, Quat, Vec3};
use glow::HasContext;
use lib76::{
    fileparsers::{
        bwd2::{WGEOLodLevel, WLOC},
        geo::Geo,
        sdf::SDF,
        vcf::VCF,
        vdf::VDF,
        vtf::VTF,
        wdf::WDF,
    },
    geo_graph::{self, GeoNode},
    virtual_fs::VirtualFS,
};

use crate::{
    caches::{TMTCache, TextureCache},
    mesh::{self, Mesh},
    RenderMode,
};

pub struct SceneNode {
    pub name: String,
    pub children: Vec<SceneNode>,
    pub local_position: Vec3,
    pub local_rotation: Quat,
    pub mesh: Option<Mesh>,
}

impl SceneNode {
    pub fn new_empty(name: String) -> Self {
        Self {
            name,
            children: Vec::new(),
            local_position: Vec3::ZERO,
            local_rotation: Quat::IDENTITY,
            mesh: None,
        }
    }

    pub fn from_sdf(
        gl: &glow::Context,
        vfs: &VirtualFS,
        sdf: &SDF,
        use_face_normals: bool,
        tmt_cache: &mut TMTCache,
    ) -> Result<Self, String> {
        let load_geo = |name: &str| -> Rc<Geo> {
            let filename = format!("{}.geo", name);
            Rc::new(vfs.load::<Geo>(&filename).expect("Failed to load geo"))
        };

        let graph = geo_graph::from(
            sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
            load_geo,
        )
        .expect("Failed to build graph");

        let mut root_node = Self::new_empty(sdf.sdfc.name.clone());
        for node in &graph {
            root_node.children.push(Self::from_geonode(
                gl,
                node,
                use_face_normals,
                &RenderMode::SGeo,
                tmt_cache,
            )?);
        }
        Ok(root_node)
    }

    pub fn from_vcf(
        gl: &glow::Context,
        vfs: &VirtualFS,
        vcf: &VCF,
        use_face_normals: bool,
        tmt_cache: &mut TMTCache<'_>,
    ) -> Result<Self, String> {
        
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

        let graph = geo_graph::from(vdf.vgeo.third_person_parts[0][0].iter(), load_geo)
            .expect("Failed to build graph");

        let render_mode = RenderMode::Vehicle(vtf);

        let mut root_node =
            Self::new_empty(format!("{} - {}", &vdf.vdfc.name, &vcf.vcfc.variant_name));

        for node in &graph {
            root_node.children.push(Self::from_geonode(
                gl,
                node,
                use_face_normals,
                &render_mode,
                tmt_cache,
            )?);
        }

        let root_node_ref = &mut root_node;

        let mut load_wheel = |filename: &str, right_wloc: &WLOC, left_wloc: &WLOC| {
            let mut from_wdf = |wgeo_lodlevel: &WGEOLodLevel, wloc: &WLOC| -> SceneNode {
                let wheel_geonode =
                    geo_graph::GeoNode::from_geopart(&wgeo_lodlevel.lod_parts[0], load_geo);
                let wheel_scene_node = SceneNode::from_geonode(
                    gl,
                    &wheel_geonode,
                    use_face_normals,
                    &RenderMode::SGeo,
                    tmt_cache,
                )
                .expect("Failed to load wheel");

                let mut wloc_scene_node = SceneNode::new_empty("Wheel".to_string());
                wloc_scene_node.local_position = wloc.position;
                wloc_scene_node.local_rotation = Quat::from_mat4(&wloc.rotation_axis.matrix);
                wloc_scene_node.children.push(wheel_scene_node);

                wloc_scene_node
            };

            let wdf: WDF = vfs.load(filename).expect("Failed to load wheel");

            let right_wheel = from_wdf(&wdf.wgeo.right[0], right_wloc);

            root_node_ref.children.push(right_wheel);

            let left_wheel = from_wdf(&wdf.wgeo.left[0], left_wloc);

            root_node_ref.children.push(left_wheel);
        };

        if vcf.vcfc.wdf_front_filename != "null" {
            load_wheel(&vcf.vcfc.wdf_front_filename, &vdf.wlocs[0], &vdf.wlocs[1]);
        }

        if vcf.vcfc.wdf_mid_filename != "null" {
            load_wheel(&vcf.vcfc.wdf_mid_filename, &vdf.wlocs[2], &vdf.wlocs[3]);
        }

        if vcf.vcfc.wdf_back_filename != "null" {
            load_wheel(&vcf.vcfc.wdf_back_filename, &vdf.wlocs[4], &vdf.wlocs[5]);
        }

        Ok(root_node)
    }

    pub fn from_geonode(
        gl: &glow::Context,
        node: &GeoNode,
        use_face_normals: bool,
        render_mode: &RenderMode,
        tmt_cache: &mut TMTCache,
    ) -> Result<Self, String> {
        let mut children = Vec::new();
        for child in &node.children {
            children.push(Self::from_geonode(
                gl,
                child,
                use_face_normals,
                render_mode,
                tmt_cache,
            )?);
        }

        Ok(Self {
            name: node.name.clone(),
            children,
            local_position: node.local_position,
            local_rotation: Quat::from_mat4(&node.axis.matrix),
            mesh: Some(Mesh::from_submeshes(
                gl,
                mesh::submeshes_from_geo(&node.geo, use_face_normals, render_mode, tmt_cache),
            )?),
        })
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        mut model_matrix: glam::Mat4,
        loc: &glow::UniformLocation,
        texture_cache: &mut TextureCache,
    ) {
        model_matrix *= glam::Mat4::from_translation(self.local_position);
        model_matrix *= glam::Mat4::from_quat(self.local_rotation);

        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(loc), false, model_matrix.as_ref());
        }

        if let Some(mesh) = &self.mesh {
            unsafe {
                gl.bind_vertex_array(Some(mesh.vao));
                for submesh in &mesh.submeshes {
                    if submesh.texture_name.is_empty() {
                        gl.bind_texture(glow::TEXTURE_2D, None);
                    } else {
                        let texture = texture_cache
                            .get(&submesh.texture_name)
                            .expect("Failed to get texture");
                        gl.bind_texture(glow::TEXTURE_2D, Some(**texture));
                    }
                    gl.draw_elements(
                        glow::TRIANGLES,
                        submesh.index_count as i32,
                        glow::UNSIGNED_SHORT,
                        submesh.index_start as i32 * std::mem::size_of::<u16>() as i32,
                    );
                }
            }
        }

        for child in &self.children {
            child.render(gl, model_matrix, loc, texture_cache);
        }
    }
}
