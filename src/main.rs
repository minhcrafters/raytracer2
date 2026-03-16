mod camera;
mod image;
mod material;
mod primitives;
mod ray;
mod utils;

use std::sync::Arc;

use crate::{
    camera::Camera,
    image::Color,
    material::{Material, dielectric::Dielectric, lambertian::Lambertian, metallic::Metallic},
    primitives::sphere::Sphere,
    ray::hittable::HittableList,
    utils::{random_f64, random_f64_range, random_vec3, random_vec3_range},
};
use glam::DVec3;
use log::info;

fn main() {
    env_logger::init();

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1200;

    let mut camera = Camera::new(aspect_ratio, image_width, 10, 50);

    camera.fov = 20.0;
    camera.look_from = DVec3::new(13.0, 2.0, 3.0);
    camera.look_at = DVec3::new(0.0, 0.0, 0.0);

    camera.defocus_angle = 0.6;
    camera.focus_dist = 10.0;

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));

    let mut world = HittableList::new();

    world.add(Box::new(Sphere::stationary(
        DVec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(ground_mat),
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_f64();
            let center = DVec3::new(
                a as f64 + 0.9 * random_f64(),
                0.2,
                b as f64 + 0.9 * random_f64(),
            );

            if (center - DVec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_mat: Arc<dyn Material>;

                if choose_mat < 0.8 {
                    let albedo = Color::from_vec3(random_vec3() * random_vec3());
                    sphere_mat = Arc::new(Lambertian::new(albedo));
                    let center2 = center + DVec3::new(0.0, random_f64_range(0.0, 0.5), 0.0);
                    world.add(Box::new(Sphere::moving(
                        center,
                        center2,
                        0.2,
                        Some(sphere_mat),
                    )));
                } else if choose_mat < 0.95 {
                    let albedo = Color::from_vec3(random_vec3_range(0.5, 1.0));
                    let fuzz = random_f64_range(0.0, 0.5);
                    sphere_mat = Arc::new(Metallic::new(albedo, fuzz));
                    world.add(Box::new(Sphere::stationary(center, 0.2, Some(sphere_mat))));
                } else {
                    sphere_mat = Arc::new(Dielectric::new(1.5));
                    world.add(Box::new(Sphere::stationary(center, 0.2, Some(sphere_mat))));
                }
            }
        }
    }

    world.add(Box::new(Sphere::stationary(
        DVec3::new(0.0, 1.0, 0.0),
        1.0,
        Some(Arc::new(Dielectric::new(1.5))),
    )));

    world.add(Box::new(Sphere::stationary(
        DVec3::new(-4.0, 1.0, 0.0),
        1.0,
        Some(Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)))),
    )));

    world.add(Box::new(Sphere::stationary(
        DVec3::new(4.0, 1.0, 0.0),
        1.0,
        Some(Arc::new(Metallic::new(Color::new(1.0, 1.0, 1.0), 0.0))),
    )));

    let image = camera.render(&world);

    let filename = format!("output.ppm");
    image.save(&filename).expect("Failed to save image");
    info!("done");
}
