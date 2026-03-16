mod camera;
pub mod hdri;
mod hittable;
mod image;
mod material;
mod optim;
mod ray;
mod utils;

use std::{f64::consts::PI, sync::Arc};

use crate::{
    camera::Camera,
    hdri::Hdri,
    hittable::{HittableList, instance::Instance, model::load_model, quad::Quad},
    image::{Color, PPMImage},
    material::{
        dielectric::Dielectric, diffuse_light::DiffuseLight, lambertian::Lambertian,
        metallic::Metallic,
    },
    optim::bvh::BvhNode,
    ray::transform::Transform,
};
use glam::{DQuat, DVec3};

fn cornell_box() -> PPMImage {
    let aspect_ratio = 1.0;
    let image_width = 600;

    let mut camera = Camera::new(aspect_ratio, image_width, 1000, 50);

    camera.fov = 40.0;
    camera.look_from = DVec3::new(278.0, 278.0, -800.0);
    camera.look_at = DVec3::new(278.0, 278.0, 0.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = hdri::Background::Hdri(Hdri::new("golden_gate_hills_2k.hdr"));

    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(15.0, 15.0, 15.0)));

    // Left wall (green)
    // world.add(Arc::new(Quad::new(
    //     DVec3::new(555.0, 0.0, 0.0),
    //     DVec3::new(0.0, 555.0, 0.0),
    //     DVec3::new(0.0, 0.0, 555.0),
    //     Some(green),
    // )));

    // Right wall (red)
    // world.add(Arc::new(Quad::new(
    //     DVec3::new(0.0, 0.0, 0.0),
    //     DVec3::new(0.0, 555.0, 0.0),
    //     DVec3::new(0.0, 0.0, 555.0),
    //     Some(red),
    // )));

    // Ceiling light
    // world.add(Arc::new(Quad::new(
    //     DVec3::new(343.0, 554.0, 332.0),
    //     DVec3::new(-130.0, 0.0, 0.0),
    //     DVec3::new(0.0, 0.0, -105.0),
    //     Some(light),
    // )));

    // Floor
    world.add(Arc::new(Quad::new(
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(555.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 555.0),
        Some(white.clone()),
    )));

    // Ceiling
    // world.add(Arc::new(Quad::new(
    //     DVec3::new(555.0, 555.0, 555.0),
    //     DVec3::new(-555.0, 0.0, 0.0),
    //     DVec3::new(0.0, 0.0, -555.0),
    //     Some(white.clone()),
    // )));

    // Back wall
    // world.add(Arc::new(Quad::new(
    //     DVec3::new(0.0, 0.0, 555.0),
    //     DVec3::new(555.0, 0.0, 0.0),
    //     DVec3::new(0.0, 555.0, 0.0),
    //     Some(white),
    // )));

    // N64 logo
    let logo_mat = Arc::new(Metallic::new(Color::new(0.8, 0.8, 0.8), 0.01));
    if let Ok(logo_mesh) = load_model("obj/n64_logo.obj", logo_mat) {
        let transform = Transform::new()
            .scale(DVec3::splat(250.0))
            .rotate(DQuat::from_rotation_y(PI / 6.0))
            .translate(DVec3::new(278.0, 0.0, 278.0));
        world.add(Arc::new(Instance::new(
            Arc::new(BvhNode::from_list(&logo_mesh)),
            transform,
        )));
    }

    let bvh_world = BvhNode::from_list(&world);
    camera.render(&bvh_world)
}

fn main() {
    env_logger::init();

    let image = cornell_box();

    image.save("output.ppm").expect("Failed to save image");

    image
        .to_rgb_image()
        .save("output.png")
        .expect("Failed to save png");

    println!("done");
}
