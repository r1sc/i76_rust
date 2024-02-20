use std::{cell::RefCell, rc::Rc};

use caches::TMTCache;
pub use glam;
pub use glow;

pub mod caches;
pub mod shader;
use lib76::fileparsers::vtf::VTF;
pub use scenenode::SceneNode;

mod cache;
mod mem_utils;
mod mesh;
mod scenenode;


pub enum RenderMode {
    SGeo,
    Vehicle(VTF)
}