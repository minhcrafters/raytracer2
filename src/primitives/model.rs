use std::sync::Arc;

use asset_importer::{Importer, postprocess::PostProcessSteps};
use glam::DVec3;

use crate::{material::Material, ray::hittable::HittableList};

use super::triangle::Triangle;

pub fn load_model(
    path: &str,
    material: Arc<dyn Material>,
) -> Result<HittableList, Box<dyn std::error::Error>> {
    let mut hittable_list = HittableList::new();

    let scene = Importer::new()
        .read_file(path)
        .with_post_process(
            PostProcessSteps::TRIANGULATE
                | PostProcessSteps::FLIP_UVS
                | PostProcessSteps::JOIN_IDENTICAL_VERTICES,
        )
        .import()?;

    for mesh in scene.meshes() {
        let vertices: Vec<_> = mesh.vertices_iter().collect();

        for face in mesh.faces() {
            let indices = face.indices();

            if indices.len() == 3 {
                let v0 = vertices[indices[0] as usize];
                let v1 = vertices[indices[1] as usize];
                let v2 = vertices[indices[2] as usize];

                let p0 = DVec3::new(v0.x as f64, v0.y as f64, v0.z as f64);
                let p1 = DVec3::new(v1.x as f64, v1.y as f64, v1.z as f64);
                let p2 = DVec3::new(v2.x as f64, v2.y as f64, v2.z as f64);

                let u = p1 - p0;
                let v = p2 - p0;

                let triangle = Triangle::new(p0, u, v, Some(material.clone()));
                hittable_list.add(Arc::new(triangle));
            }
        }
    }

    Ok(hittable_list)
}
