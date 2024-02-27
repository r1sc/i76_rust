use std::rc::Rc;

use glam::{Quat, Vec3};
use glow::HasContext;
use lib76::{
    fileparsers::{
        bwd2::{WGEOLodLevel, WLOC},
        geo::Geo,
        sdf::SDF,
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

pub struct SceneNodeLoaderParams<'a, 'b> {
    pub gl: &'a glow::Context,
    pub vfs: &'a VirtualFS,
    pub use_face_normals: bool,
    pub tmt_cache: &'a mut TMTCache<'b>,
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
        loader_params: &mut SceneNodeLoaderParams<'_, '_>,
        sdf: &SDF,
    ) -> Result<Self, String> {
        let load_geo = |name: &str| -> Rc<Geo> {
            let filename = format!("{}.geo", name);
            Rc::new(
                loader_params
                    .vfs
                    .load::<Geo>(&filename)
                    .expect("Failed to load geo"),
            )
        };

        let graph = geo_graph::from(
            sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
            load_geo,
        )
        .expect("Failed to build graph");

        let mut root_node = Self::new_empty(sdf.sdfc.name.clone());
        for node in &graph {
            root_node
                .children
                .push(Self::from_geonode(loader_params, node, &RenderMode::SGeo)?);
        }
        Ok(root_node)
    }

    pub fn from_wdf(
        loader_params: &mut SceneNodeLoaderParams<'_, '_>,
        wgeo_lodlevel: &WGEOLodLevel,
        wloc: &WLOC,
    ) -> Result<Self, String> {
        let load_geo = |name: &str| -> Rc<Geo> {
            let filename = format!("{}.geo", name);
            Rc::new(
                loader_params
                    .vfs
                    .load::<Geo>(&filename)
                    .expect("Failed to load geo"),
            )
        };

        let wheel_geonode = geo_graph::GeoNode::from_geopart(&wgeo_lodlevel.lod_parts[0], load_geo);
        let wheel_scene_node =
            Self::from_geonode(loader_params, &wheel_geonode, &RenderMode::SGeo)?;

        let mut wloc_scene_node = SceneNode::new_empty("Wheel".to_string());
        wloc_scene_node.local_position = wloc.position;
        wloc_scene_node.local_rotation = Quat::from_mat4(&wloc.rotation_axis.matrix);
        wloc_scene_node.children.push(wheel_scene_node);

        Ok(wloc_scene_node)
    }

    pub fn from_car(
        loader_params: &mut SceneNodeLoaderParams<'_, '_>,
        car: &lib76::car_loader::CarParts,
        damage_state: u32,
        lod_level: u32,
    ) -> Result<Self, String> {
        let render_mode = RenderMode::Vehicle(&car.vtf);

        let mut root_node = Self::new_empty(format!(
            "{} - {}",
            &car.vdf.vdfc.name, &car.vcf.vcfc.variant_name
        ));

        for node in &car.lods[lod_level as usize].damage_state_graphs[damage_state as usize] {
            root_node
                .children
                .push(Self::from_geonode(loader_params, node, &render_mode)?);
        }

        let root_node_ref = &mut root_node;

        let mut load_wheel =
            |filename: &str, right_wloc: &WLOC, left_wloc: &WLOC| -> Result<(), String> {
                let wdf: WDF = loader_params
                    .vfs
                    .load(filename)
                    .expect("Failed to load wheel");

                let right_wheel =
                    Self::from_wdf(loader_params, &wdf.wgeo.right[lod_level as usize], right_wloc)?;

                root_node_ref.children.push(right_wheel);

                let left_wheel =
                    Self::from_wdf(loader_params, &wdf.wgeo.left[lod_level as usize], left_wloc)?;

                root_node_ref.children.push(left_wheel);

                Ok(())
            };

        if car.vcf.vcfc.wdf_front_filename != "null" {
            load_wheel(&car.vcf.vcfc.wdf_front_filename, &car.vdf.wlocs[0], &car.vdf.wlocs[1])
                .expect("Failed to load front wheels");
        }

        if car.vcf.vcfc.wdf_mid_filename != "null" {
            load_wheel(&car.vcf.vcfc.wdf_mid_filename, &car.vdf.wlocs[2], &car.vdf.wlocs[3])
                .expect("Failed to load mid wheels");
        }

        if car.vcf.vcfc.wdf_back_filename != "null" {
            load_wheel(&car.vcf.vcfc.wdf_back_filename, &car.vdf.wlocs[4], &car.vdf.wlocs[5])
                .expect("Failed to load back wheels");
        }

        Ok(root_node)
    }

    pub fn from_geonode(
        loader_params: &mut SceneNodeLoaderParams<'_, '_>,
        node: &GeoNode,
        render_mode: &RenderMode,
    ) -> Result<Self, String> {
        let mut children = Vec::new();
        for child in &node.children {
            children.push(Self::from_geonode(loader_params, child, render_mode)?);
        }

        Ok(Self {
            name: node.name.clone(),
            children,
            local_position: node.local_position,
            local_rotation: Quat::from_mat4(&node.axis.matrix),
            mesh: Some(Mesh::from_submeshes(
                loader_params.gl,
                mesh::submeshes_from_geo(
                    &node.geo,
                    loader_params.use_face_normals,
                    render_mode,
                    loader_params.tmt_cache,
                ),
            )?),
        })
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        mut model_matrix: glam::Mat4,
        view_matrix: glam::Mat4,
        u_modelview: &glow::UniformLocation,
        u_normal: &glow::UniformLocation,
        texture_cache: &mut TextureCache,
    ) {
        model_matrix *= glam::Mat4::from_translation(self.local_position);
        model_matrix *= glam::Mat4::from_quat(self.local_rotation);

        unsafe {
            let modelview = view_matrix * model_matrix;
            gl.uniform_matrix_4_f32_slice(Some(u_modelview), false, modelview.as_ref());

            let normalmatrix = modelview.inverse().transpose();
            gl.uniform_matrix_4_f32_slice(Some(u_normal), false, normalmatrix.as_ref());
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
            child.render(
                gl,
                model_matrix,
                view_matrix,
                u_modelview,
                u_normal,
                texture_cache,
            );
        }
    }
}
