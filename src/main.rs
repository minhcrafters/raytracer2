mod camera;
mod image;
mod material;
mod ray;
mod utils;

use std::{
    f64::{EPSILON, INFINITY, consts::PI},
    sync::Arc,
};

use crate::{
    camera::Camera,
    image::{Color, PPMImage},
    material::{lambertian::Lambertian, metallic::Metallic},
    ray::{
        Ray,
        hit::{Hittable, HittableList, Sphere},
        interval::Interval,
    },
};
use glam::DVec3;
use log::info;

fn ray_color(r: &Ray, world: &impl Hittable) -> Color {
    if let Some(rec) = world.hit(r, &Interval::new(0.0, INFINITY)) {
        return Color::from_vec3(rec.normal + 1.0) * 0.5;
    }

    let unit_dir = r.dir.normalize();
    let a = 0.5 * (unit_dir.y + 1.0);
    let color = (1.0 - a) * DVec3::ONE + a * DVec3::new(0.5, 0.7, 1.0);
    Color::from_vec3(color)
}

fn main() {
    env_logger::init();

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;

    let mut camera = Camera::new(aspect_ratio, image_width, 100, 50);
    let mut world = HittableList::new();

    world.add(Box::new(Sphere {
        center: DVec3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: Some(Arc::new(Metallic::new(Color::new(255.0, 255.0, 255.0)))),
    }));

    world.add(Box::new(Sphere {
        center: DVec3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: Some(Arc::new(Lambertian::new(Color::new(255.0, 255.0, 255.0)))),
    }));

    let image = camera.render(&world);

    let filename = format!("output.ppm");
    image.save(&filename).expect("Failed to save image");
    info!("done");
}
