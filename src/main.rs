mod image;
mod ray;
mod viewport;

use crate::{
    image::{Color, PPMImage},
    ray::Ray,
    viewport::Viewport,
};
use glam::DVec3;
use log::{error, info};

fn ray_color(r: &Ray) -> Color {
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
    let viewport_width = viewport_height * (image_width / image_height) as f64;
    let focal_length = 1.0;

    let viewport = Viewport {
        width: viewport_width,
        height: viewport_height,
        focal_length,
    };

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
