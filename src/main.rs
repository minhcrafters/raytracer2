mod camera;
mod image;
mod material;
mod optim;
mod primitives;
mod ray;
mod utils;

use std::sync::Arc;

use crate::{
    camera::Camera,
    image::Color,
    material::{diffuse_light::DiffuseLight, lambertian::Lambertian},
    optim::bvh::BvhNode,
    primitives::quad::Quad,
    ray::hittable::HittableList,
};
use glam::DVec3;
use log::info;

fn main() {
    env_logger::init();

    let aspect_ratio = 1.0;
    let image_width = 600;

    let mut camera = Camera::new(aspect_ratio, image_width, 50, 50);

    camera.fov = 40.0;
    camera.look_from = DVec3::new(278.0, 278.0, -800.0);
    camera.look_at = DVec3::new(278.0, 278.0, 0.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = Color::new(0.0, 0.0, 0.0);

    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(15.0, 15.0, 15.0)));

    // Left wall (green)
    world.add(Arc::new(Quad::new(
        DVec3::new(555.0, 0.0, 0.0),
        DVec3::new(0.0, 555.0, 0.0),
        DVec3::new(0.0, 0.0, 555.0),
        Some(green),
    )));

    // Right wall (red)
    world.add(Arc::new(Quad::new(
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(0.0, 555.0, 0.0),
        DVec3::new(0.0, 0.0, 555.0),
        Some(red),
    )));

    // Ceiling light
    world.add(Arc::new(Quad::new(
        DVec3::new(343.0, 554.0, 332.0),
        DVec3::new(-130.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, -105.0),
        Some(light),
    )));

    // Floor
    world.add(Arc::new(Quad::new(
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(555.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 555.0),
        Some(white.clone()),
    )));

    // Ceiling
    world.add(Arc::new(Quad::new(
        DVec3::new(555.0, 555.0, 555.0),
        DVec3::new(-555.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, -555.0),
        Some(white.clone()),
    )));

    // Back wall
    world.add(Arc::new(Quad::new(
        DVec3::new(0.0, 0.0, 555.0),
        DVec3::new(555.0, 0.0, 0.0),
        DVec3::new(0.0, 555.0, 0.0),
        Some(white),
    )));

    let bvh_world = BvhNode::from_list(&world);
    let image = camera.render(&bvh_world);

    let filename = format!("output.ppm");
    image.save(&filename).expect("Failed to save image");
    println!("done");
}
