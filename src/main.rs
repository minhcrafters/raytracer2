mod image;
mod ray;

use crate::{
    image::{Color, PPMImage},
    ray::Ray,
};
use glam::DVec3;
use log::{error, info};

fn hit_sphere(center: DVec3, radius: f64, ray: &Ray) -> bool {
    let oc = center - ray.origin;
    let a = ray.dir.dot(ray.dir);
    let b = -2.0 * ray.dir.dot(oc);
    let c = oc.dot(oc) - radius * radius;
    let dis = b * b - 4.0 * a * c;
    dis >= 0.0
}

fn ray_color(r: &Ray) -> Color {
    if hit_sphere(DVec3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Color::new(255, 0, 0);
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

    let mut image_height = (image_width as f64 / aspect_ratio) as usize;
    image_height = if image_height < 1 { 1 } else { image_height };

    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let focal_length = 1.0;

    let camera_center = DVec3::new(0.0, 0.0, 0.0);
    let viewport_u = DVec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = DVec3::new(0.0, -viewport_height, 0.0);

    let pixel_delta_u = viewport_u / image_width as f64;
    let pixel_delta_v = viewport_v / image_height as f64;

    let viewport_upper_left =
        camera_center - DVec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let mut image = PPMImage::new(image_width, image_height as usize);
    for y in 0..image.height {
        for x in 0..image.width {
            info!("Scanlines remaining: {}", image.height - y);

            let pixel_center =
                pixel00_loc + (x as f64 * pixel_delta_u) + (y as f64 * pixel_delta_v);
            let ray_dir = pixel_center - camera_center;

            let r = Ray::new(camera_center, ray_dir);

            let color = ray_color(&r);
            image.set_pixel(x, y, &color);
        }
    }
    let filename = format!("output.ppm");
    image.save(&filename).expect("Failed to save image");
    info!("done");
}
