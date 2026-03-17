mod background;
mod camera;
mod hittable;
mod image;
mod material;
mod optim;
mod pdf;
mod ray;
mod utils;

use std::sync::Arc;

use crate::{
    background::{Background, hdri::Hdri},
    camera::Camera,
    hittable::{HittableList, model::TriMesh, quad::Quad},
    image::{Color, PPMImage},
    material::{
        dielectric::Dielectric, diffuse_light::DiffuseLight, lambertian::Lambertian,
        metallic::Metallic,
    },
    optim::bvh::BvhNode,
    ray::transform::Transform,
};
use glam::DVec3;

fn cornell_box() -> PPMImage {
    let aspect_ratio = 1.0;
    let image_width = 1200;

    let mut camera = Camera::new(aspect_ratio, image_width, 500, 50);

    camera.fov = 40.0;
    camera.look_from = DVec3::new(278.0, 278.0, -800.0);
    camera.look_at = DVec3::new(278.0, 278.0, 0.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = Background::Hdri(Hdri::new("glasshouse_interior_4k.hdr"));

    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::new(Color::new(15.0, 15.0, 15.0)));

    let floor = Quad::new(
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(555.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 555.0),
        Some(white.clone()),
    );
    world.add(Arc::new(floor));

    let back_wall = Quad::new(
        DVec3::new(0.0, 0.0, 555.0),
        DVec3::new(555.0, 0.0, 0.0),
        DVec3::new(0.0, 555.0, 0.0),
        Some(white),
    );
    world.add(Arc::new(back_wall));

    let obj_mat = Arc::new(Metallic::new(Color::new(1.0, 1.0, 1.0), 0.1));
    let transform = Transform::new()
        .scale(DVec3::splat(80.0))
        .translate(DVec3::new(278.0, 0.0, 278.0));

    if let Ok(meshes) = TriMesh::load_model("obj/teapot.obj", obj_mat, true) {
        for mut mesh in meshes {
            mesh.transform = transform;
            world.add(Arc::new(mesh));
        }
    }

    let bvh_world = BvhNode::from_list(&world);
    camera.render(&bvh_world, &lights)
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
