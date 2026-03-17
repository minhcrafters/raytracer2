mod background;
mod camera;
mod hittable;
mod image;
mod material;
mod optim;
mod pdf;
mod ray;
mod utils;

use std::{
    f64::consts::{FRAC_PI_2, FRAC_PI_6, PI},
    sync::Arc,
};

use crate::{
    background::{Background, hdri::Hdri},
    camera::Camera,
    hittable::{HittableList, model::TriMesh, quad::Quad, sphere::Sphere},
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
    let image_width = 1200;

    let mut camera = Camera::new(aspect_ratio, image_width, 500, 50);

    camera.fov = 40.0;
    camera.look_from = DVec3::new(278.0, 278.0, -800.0);
    camera.look_at = DVec3::new(278.0, 278.0, 0.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = Background::Hdri(Hdri::new("hdri/glasshouse_interior_4k.hdr"));

    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::new(Color::new(15.0, 15.0, 15.0)));

    let mut floor = Quad::new();
    floor.material = Some(white.clone());
    floor.transform = Transform::new()
        .scale(DVec3::new(555.0, 555.0, 1.0))
        .rotate(DQuat::from_rotation_x(FRAC_PI_2))
        .translate(DVec3::new(555.0 / 2.0, 0.0, 555.0 / 2.0));
    world.add(Arc::new(floor));

    let mut back_wall = Quad::new();
    back_wall.material = Some(white);
    back_wall.transform = Transform::new()
        .scale(DVec3::new(555.0, 555.0, 1.0))
        .translate(DVec3::new(555.0 / 2.0, 555.0 / 2.0, 555.0));
    world.add(Arc::new(back_wall));

    let obj_mat = Arc::new(Dielectric::new(Color::new(1.0, 1.0, 1.0), 1.5, 0.0));
    let transform = Transform::new()
        .scale(DVec3::splat(85.0))
        .rotate(DQuat::from_rotation_y(PI))
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

fn dragon() -> PPMImage {
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 600;

    let mut camera = Camera::new(aspect_ratio, image_width, 500, 50);

    camera.fov = FRAC_PI_6.to_degrees();
    camera.look_from = DVec3::new(-2.5, 4.0, 6.5);
    camera.look_at = DVec3::new(0.0, 0.0, 0.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = Background::Color(Color::new(0.1, 0.1, 0.1));

    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    let white = Arc::new(Lambertian::new(Color::from_hex(0xAAAAAA)));
    let light_mat = Arc::new(DiffuseLight::new(Color::from_hex(0xFFAAAA) * 20.0));

    let mut floor = Quad::new();
    floor.material = Some(white.clone());
    floor.transform = Transform::new()
        .scale(DVec3::new(5.0, 5.0, 1.0))
        .rotate(DQuat::from_rotation_x(FRAC_PI_2));
    world.add(Arc::new(floor));

    let mut sphere = Sphere::new();
    sphere.material = Some(light_mat);
    sphere.transform = Transform::new()
        .scale(DVec3::splat(2.0))
        .translate(DVec3::new(0.0, 20.0, 3.0));
    let sphere = Arc::new(sphere);
    world.add(sphere.clone());
    lights.add(sphere);

    let obj_mat = Arc::new(Lambertian::new(Color::from_hex(0xB7CA79)));
    let transform = Transform::new()
        .scale(DVec3::splat(3.4))
        .rotate(DQuat::from_rotation_y(FRAC_PI_2));

    if let Ok(meshes) = TriMesh::load_model("obj/dragon.obj", obj_mat, true) {
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

    let image = dragon();

    image.save("output.ppm").expect("Failed to save image");

    image
        .to_rgb_image()
        .save("output.png")
        .expect("Failed to save png");

    println!("done");
}
