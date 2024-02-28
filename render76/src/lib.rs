pub use glam;
pub use glow;

pub mod caches;
pub mod shader;
use lib76::fileparsers::vtf::VTF;
pub use scenenode::{SceneNode, SceneNodeLoaderParams};

mod cache;
mod mem_utils;
mod mesh;
mod scenenode;
mod terrain;

pub enum RenderMode<'a> {
    SGeo,
    Vehicle(&'a VTF)
}
