use std::rc::Rc;

use glam::{Quat, Vec3};
use glow::HasContext;
use lib76::{
    fileparsers::{geo::Geo, sdf::SDF},
    geo_graph::{self, GeoNode},
    virtual_fs::VirtualFS,
};

use crate::{
    caches::TextureCache,
    mesh::{self, Mesh},
};

pub struct SceneNode {
    pub name: String,
    pub children: Vec<SceneNode>,
    pub local_position: Vec3,
    pub local_rotation: Quat,
    pub mesh: Option<Mesh>,
}

impl SceneNode {
    pub fn from_sdf(
        gl: &glow::Context,
        vfs: &VirtualFS,
        sdf: &SDF,
        use_face_normals: bool,
    ) -> Result<Vec<Self>, String> {
        let load_geo = |name: &str| -> Rc<Geo> {
            let filename = format!("{}.geo", name);
            Rc::new(vfs.load::<Geo>(&filename).expect("Failed to load geo"))
        };

        let graph = geo_graph::from(
            sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
            load_geo,
        )
        .expect("Failed to build graph");

        let mut root_nodes = Vec::new();
        for node in &graph {
            root_nodes.push(Self::from_geonode(gl, node, use_face_normals)?);
        }
        Ok(root_nodes)
    }

    pub fn from_geonode(
        gl: &glow::Context,
        node: &GeoNode,
        use_face_normals: bool,
    ) -> Result<Self, String> {
        let mut children = Vec::new();
        for child in &node.children {
            children.push(Self::from_geonode(gl, child, use_face_normals)?);
        }

        Ok(Self {
            name: node.name.clone(),
            children,
            local_position: node.local_position,
            local_rotation: Quat::from_mat4(&node.axis.matrix),
            mesh: Some(Mesh::from_submeshes(
                gl,
                mesh::submeshes_from_geo(&node.geo, use_face_normals),
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
