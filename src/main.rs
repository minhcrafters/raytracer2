pub mod background;
pub mod camera;
pub mod gpu;
pub mod hittable;
pub mod image;
pub mod material;
pub mod optim;
pub mod pdf;
pub mod ray;
pub mod texture;
pub mod utils;

use std::{
    f64::consts::{FRAC_PI_2, PI},
    sync::Arc,
};

use crate::{
    background::{Background, hdri::Hdri},
    camera::Camera,
    gpu::{renderer::GpuRenderer, serialize::SceneSerializer},
    hittable::{HittableList, cuboid::Cuboid, model::TriMesh, quad::Quad, sphere::Sphere},
    image::{Color, PPMImage},
    material::{
        dielectric::Dielectric, diffuse_light::DiffuseLight, lambertian::Lambertian,
        metallic::Metallic, specular::Specular,
    },
    optim::bvh::BvhNode,
    ray::transform::Transform,
};
use glam::{DQuat, DVec3};

#[allow(dead_code)]
fn cornell_box() -> PPMImage {
    let aspect_ratio = 1.0;
    let image_width = 1024;
    let spp = 500;

    let mut camera = Camera::new(aspect_ratio, image_width, spp, 50);

    camera.fov = 0.686 * 180.0 / std::f64::consts::PI;
    camera.look_from = DVec3::new(278.0, 273.0, -800.0);
    camera.look_at = camera.look_from + DVec3::new(0.0, 0.0, 1.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = Background::Hdri(Hdri::new("hdri/german_town_street_4k.hdr"));

    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    let white = Arc::new(Lambertian::new(Color::from_hex(0xAAAAAA)));
    let red = Arc::new(Lambertian::new(Color::from_hex(0xBC0000)));
    let green = Arc::new(Lambertian::new(Color::from_hex(0x00BC00)));
    let light_mat = Arc::new(DiffuseLight::new(Color::from_hex(0xFFFFFF) * 100.0));

    let box1_mat = Arc::new(Metallic::new(Color::from_hex(0xFFFFFF), 0.0));
    let box2_mat = Arc::new(Lambertian::new(Color::new(1.0, 1.0, 1.0)));

    let floor_q = DVec3::new(0.0, 0.0, 0.0);
    let floor_u = DVec3::new(556.0, 0.0, 0.0);
    let floor_v = DVec3::new(0.0, 0.0, 559.2);
    world.add(Arc::new(Quad::from_points(
        floor_q,
        floor_u,
        floor_v,
        Some(white.clone()),
    )));

    let ceiling_q = DVec3::new(0.0, 548.9, 0.0);
    let ceiling_u = DVec3::new(556.0, 0.0, 0.0);
    let ceiling_v = DVec3::new(0.0, 0.0, 559.2);
    world.add(Arc::new(Quad::from_points(
        ceiling_q,
        ceiling_u,
        ceiling_v,
        Some(white.clone()),
    )));

    let back_q = DVec3::new(0.0, 0.0, 559.2);
    let back_u = DVec3::new(556.0, 0.0, 0.0);
    let back_v = DVec3::new(0.0, 548.9, 0.0);
    world.add(Arc::new(Quad::from_points(
        back_q,
        back_u,
        back_v,
        Some(white.clone()),
    )));

    let right_q = DVec3::new(0.0, 0.0, 0.0);
    let right_u = DVec3::new(0.0, 0.0, 559.2);
    let right_v = DVec3::new(0.0, 548.9, 0.0);
    world.add(Arc::new(Quad::from_points(
        right_q,
        right_u,
        right_v,
        Some(green),
    )));

    let left_q = DVec3::new(556.0, 0.0, 0.0);
    let left_u = DVec3::new(0.0, 0.0, 559.2);
    let left_v = DVec3::new(0.0, 548.9, 0.0);
    world.add(Arc::new(Quad::from_points(
        left_q,
        left_u,
        left_v,
        Some(red),
    )));

    let light_q = DVec3::new(213.0, 548.8, 227.0);
    let light_u = DVec3::new(130.0, 0.0, 0.0);
    let light_v = DVec3::new(0.0, 0.0, 105.0);
    let light_quad = Arc::new(Quad::from_points(
        light_q,
        light_u,
        light_v,
        Some(light_mat),
    ));
    world.add(light_quad.clone());
    lights.add(light_quad);

    let mut large_box = Cuboid::new(
        DVec3::splat(-0.5),
        DVec3::splat(0.5),
        Some(box1_mat.clone()),
    );
    large_box.transform = Transform::new()
        .scale(DVec3::new(165.0, 330.0, 165.0))
        .rotate(DQuat::from_rotation_y(PI * 2.0 * (-253.0 / 360.0)))
        .translate(DVec3::new(368.0, 165.0, 351.0));
    world.add(Arc::new(large_box));

    if let Ok(meshes) = TriMesh::load_model("obj/n64_logo.obj", box2_mat.clone(), false) {
        for mut mesh in meshes {
            mesh.transform = Transform::new()
                .scale(DVec3::new(165.0, 165.0, 165.0))
                .rotate(DQuat::from_rotation_y(PI * 2.0 * (-197.0 / 360.0)))
                .translate(DVec3::new(185.0, 82.5, 169.0));
            world.add(Arc::new(mesh));
        }
    }

    let bvh_world = BvhNode::from_list(&world);
    render_gpu(&mut camera, &world, &lights, &bvh_world, spp as u32)
}

#[allow(dead_code)]
fn teapot_hdri() -> PPMImage {
    let aspect_ratio = 1.0;
    let image_width = 1200;
    let spp = 500;

    let mut camera = Camera::new(aspect_ratio, image_width, spp, 50);

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
        .scale(DVec3::new(555.0, 1.0, 555.0))
        .translate(DVec3::new(555.0 / 2.0, 0.0, 555.0 / 2.0));
    world.add(Arc::new(floor));

    let mut back_wall = Quad::new();
    back_wall.material = Some(white);
    back_wall.transform = Transform::new()
        .scale(DVec3::new(555.0, 1.0, 555.0))
        .rotate(DQuat::from_rotation_x(FRAC_PI_2))
        .translate(DVec3::new(555.0 / 2.0, 555.0 / 2.0, 555.0));
    world.add(Arc::new(back_wall));

    let obj_mat = Arc::new(Dielectric::new(Color::new(1.0, 1.0, 1.0), 1.5, 0.0));
    let transform = Transform::new()
        .scale(DVec3::splat(85.0))
        .translate(DVec3::new(278.0, 0.0, 278.0));

    if let Ok(meshes) = TriMesh::load_model("obj/teapot.obj", obj_mat.clone(), true) {
        for mut mesh in meshes {
            mesh.transform = transform;
            mesh.material = obj_mat.clone();
            world.add(Arc::new(mesh));
        }
    }

    let bvh_world = BvhNode::from_list(&world);
    render_gpu(&mut camera, &world, &lights, &bvh_world, spp as u32)
}

#[allow(dead_code)]
fn dragon() -> PPMImage {
    let aspect_ratio = 4.0 / 3.0;
    let image_width = 1200;
    let spp = 500;

    let mut camera = Camera::new(aspect_ratio, image_width, spp, 50);

    camera.fov = 22.0;
    camera.look_from = DVec3::new(-2.5, 4.0, 6.5);
    camera.look_at = DVec3::new(0.0, 0.0, 0.0);
    camera.vup = DVec3::new(0.0, 1.0, 0.0);

    camera.defocus_angle = 0.0;
    camera.background = Background::Color(Color::new(0.01, 0.01, 0.01));

    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    let white = Arc::new(Lambertian::new(Color::from_hex(0xAAAAAA)));
    let light1_mat = Arc::new(DiffuseLight::new(Color::new(1.0, 1.0, 1.0) * 320.0));
    let light2_mat = Arc::new(DiffuseLight::new(Color::from_hex(0xFFAAAA) * 800.0));

    let mut floor = Quad::new();
    floor.material = Some(white.clone());
    floor.transform = Transform::new()
        .scale(DVec3::new(50.0, 1.0, 50.0))
        .translate(DVec3::new(0.0, -1.0, 0.0));
    world.add(Arc::new(floor));

    let mut sphere = Sphere::new();
    sphere.material = Some(light1_mat);
    sphere.transform = Transform::new()
        .scale(DVec3::splat(2.0))
        .translate(DVec3::new(0.0, 20.0, 3.0));
    let sphere = Arc::new(sphere);
    world.add(sphere.clone());
    lights.add(sphere);

    let mut sphere = Sphere::new();
    sphere.material = Some(light2_mat);
    sphere.transform = Transform::new()
        .scale(DVec3::splat(0.05))
        .translate(DVec3::new(-1.0, 0.71, 0.0));
    let sphere = Arc::new(sphere);
    world.add(sphere.clone());
    lights.add(sphere);

    let obj_mat = Arc::new(Specular::new(Color::from_hex(0xa2c91c), 1.5, 10000.0));
    let transform = Transform::new()
        .scale(DVec3::splat(3.4))
        .rotate(DQuat::from_rotation_y(FRAC_PI_2));

    if let Ok(meshes) = TriMesh::load_model("obj/dragon.obj", obj_mat.clone(), true) {
        for mut mesh in meshes {
            mesh.transform = transform;
            mesh.material = obj_mat.clone();
            world.add(Arc::new(mesh));
        }
    }

    let bvh_world = BvhNode::from_list(&world);
    render_gpu(&mut camera, &world, &lights, &bvh_world, spp as u32)
}

fn render_gpu(
    camera: &mut Camera,
    world: &HittableList,
    lights: &HittableList,
    bvh: &BvhNode,
    spp: u32,
) -> PPMImage {
    let mut serializer = SceneSerializer::new();
    let mut scene = serializer.serialize(camera, world, lights, bvh);

    if let Background::Hdri(ref hdri) = camera.background {
        let (pixels, width, height) = hdri.get_data();
        scene.camera.hdri_width = width as u32;
        scene.camera.hdri_height = height as u32;
        scene.hdri_pixels = pixels
            .iter()
            .map(|c| [c.r as f32, c.g as f32, c.b as f32, 0.0])
            .collect();
    }

    let renderer = GpuRenderer::new();
    renderer.render(&scene, spp)
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
