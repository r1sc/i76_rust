use std::rc::Rc;

pub use glam;
use glam::{Quat, Vec3};
pub use glow;

use lib76::{
    fileparsers::{geo::Geo, sdf::SDF},
    geo_graph::{self, GeoNode},
    virtual_fs::VirtualFS,
};

mod cache;
pub mod caches;
mod mem_utils;
mod mesh;
pub mod shader;

pub struct SceneNode {
    pub name: String,
    pub children: Vec<SceneNode>,
    pub local_position: Vec3,
    pub local_rotation: Quat,
    pub mesh: Option<mesh::Mesh>,
}

fn build_scene_nodes(gl: &glow::Context, graph: &[GeoNode], use_face_normals: bool) -> Result<Vec<SceneNode>, String> {
    let mut scene_nodes = Vec::new();

    for node in graph {
        let mut children = Vec::new();
        for child in &node.children {
            for node in build_scene_nodes(gl, &[child.clone()], use_face_normals)? {
                children.push(node);
            }
        }

        let scene_node = SceneNode {
            name: node.name.clone(),
            children,
            local_position: node.local_position,
            local_rotation: Quat::from_mat4(&node.axis.matrix),
            mesh: Some(mesh::Mesh::from_submeshes(
                gl,
                mesh::submeshes_from_geo(&node.geo, use_face_normals),
            )?),
        };
        scene_nodes.push(scene_node);
    }

    Ok(scene_nodes)
}

pub fn build_static_sdf(
    gl: &glow::Context,
    vfs: &VirtualFS,
    sdf: &SDF,
    use_face_normals: bool
) -> Result<Vec<SceneNode>, String> {
    let load_geo = |name: &str| -> Rc<Geo> {
        let filename = format!("{}.geo", name);
        Rc::new(vfs.load::<Geo>(&filename).expect("Failed to load geo"))
    };

    let graph = geo_graph::from(
        sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
        load_geo,
    )
    .expect("Failed to build graph");

    let scene_nodes = build_scene_nodes(gl, &graph, use_face_normals)?;
    Ok(scene_nodes)
}
