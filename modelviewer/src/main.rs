// #![windows_subsystem = "windows"]
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;

use kiss3d::light::Light;
use kiss3d::nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use lib76::fileparsers::binary_reader::{BinaryReader, Readable};
use lib76::fileparsers::geo::Geo;
use lib76::fileparsers::map::MAP;
use lib76::fileparsers::sdf::SDF;
use lib76::fileparsers::vqm::VQM;
use lib76::render_graph::{self, GeoNode};

type SubmeshTriangle = Point3<u16>;

fn add_geo_mesh(geo: &Geo, parent: &mut SceneNode, folder: &str) -> SceneNode {
    let coords: Vec<_> = geo
        .vertices
        .iter()
        .map(|v| Point3::new(v.x, v.y, v.z))
        .collect();
    let normals: Vec<_> = geo
        .normals
        .iter()
        .map(|n| Vector3::new(-n.x, -n.y, -n.z))
        .collect();

    let mut faces_by_texture_name: HashMap<String, Vec<SubmeshTriangle>> = HashMap::new();
    for face in &geo.faces {
        let submesh = faces_by_texture_name.entry(face.texture_name.clone()).or_insert(Vec::new());

        let vrefs = face.vertex_refs.as_slice();
        let first = &vrefs[0];
        for v in vrefs[1..].windows(2) {
            submesh.push(Point3::new(
                v[1].vertex_index as u16,
                v[0].vertex_index as u16,
                first.vertex_index as u16,
            ));
        }
    }

    for texture_name in faces_by_texture_name.keys() {
        let path = format!("{}/{}", folder, texture_name);
        

        println!("Need to load {}",path);
        
        // let mut reader = BinaryReader {
        //     reader: BufReader::new(Box::new(
        //         File::open(&path).unwrap_or_else(|_| panic!("Failed to open path {}", &path)),
        //     )),
        // };
        // let extension = Path::from(path).extension().unwrap().to_str().unwrap();
        // if extension == "MAP" {
        //     Rc::new(MAP::consume(&mut reader).expect("Failed to load MAP")).to_rgba_pixels(act, upside_down)
        // } else {
        //     Rc::new(VQM::consume(&mut reader).expect("Failed to load MAP")).to_rgba_pixels(cbk, act, upside_down)
        // }

    }

    let mut g = parent.add_group();
    for (texture_name, faces) in faces_by_texture_name {
        let mut mesh = g.add_mesh(
            Rc::new(RefCell::new(Mesh::new(
                coords.clone(),
                faces,
                Some(normals.clone()),
                None,
                false,
            ))),
            Vector3::new(1.0, 1.0, 1.0),
        );
        mesh.set_texture_with_name(&texture_name);
    }   

    g
}

fn add_geonode(graph: Vec<GeoNode>, scenenode: &mut SceneNode, folder: &str) {
    for node in graph {
        let mut scene_mesh = add_geo_mesh(&node.geo, scenenode, folder);

        scene_mesh.prepend_to_local_rotation(&UnitQuaternion::from_basis_unchecked(&[
            Vector3::new(node.axis.right.x, node.axis.right.y, node.axis.right.z),
            Vector3::new(node.axis.up.x, node.axis.up.y, node.axis.up.z),
            Vector3::new(
                node.axis.forward.x,
                node.axis.forward.y,
                node.axis.forward.z,
            ),
        ]));

        scene_mesh.prepend_to_local_translation(&Translation3::new(
            node.local_position.x,
            node.local_position.y,
            node.local_position.z,
        ));

        add_geonode(node.children, &mut scene_mesh, folder);
    }
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        println!("usage: modelviewer <path>");
        return Ok(());
    }

    let path = Path::new(&args[1]);
    let filename = path.file_name().unwrap().to_str().unwrap();
    let folder = path.parent().unwrap().to_str().unwrap();
    let extension = path.extension().unwrap().to_str().unwrap();

    let load_geo = |name: &str| -> Rc<Geo> {
        let path = format!(r"{}/{}.geo", folder, name);
        let mut reader = BinaryReader {
            reader: BufReader::new(Box::new(
                File::open(&path).unwrap_or_else(|_| panic!("Failed to open path {}", &path)),
            )),
        };
        Rc::new(Geo::consume(&mut reader).expect("Failed to load geo"))
    };

    let mut window = Window::new("modelviewer");

    if extension == "geo" {
        let geo = load_geo(filename);
        add_geo_mesh(&geo, window.scene_mut(), folder);        
    } else if extension == "sdf" {
        let mut reader = BinaryReader {
            reader: BufReader::new(Box::new(File::open(path)?)),
        };
        let sdf = SDF::consume(&mut reader)?;
        let graph = render_graph::from(
            sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
            load_geo,
        )?;
        let mut g = window.add_group();
        add_geonode(graph, &mut g, folder);
    }

    window.set_light(Light::StickToCamera);

    // let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        // c.prepend_to_local_rotation(&rot);
    }

    Ok(())
}
