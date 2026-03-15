use std::f64::EPSILON;

use glam::DVec3;
use log::info;

use crate::{
    image::{Color, PPMImage},
    ray::{
        Ray,
        hit::{self, Hittable},
        interval::Interval,
    },
    utils::{random_f64, random_on_hemisphere, random_unit_vec3},
};

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub spp: usize,
    pub max_depth: usize,

    image_height: usize,
    center: DVec3,
    pixel_delta_u: DVec3,
    pixel_delta_v: DVec3,
    pixel00_loc: DVec3,
    pixel_samples_scale: f64,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: usize, spp: usize, max_depth: usize) -> Self {
        Self {
            aspect_ratio,
            image_width,
            spp,
            max_depth,
            image_height: 0,
            pixel_samples_scale: 1.0 / (spp as f64),
            center: DVec3::ZERO,
            pixel_delta_u: DVec3::ZERO,
            pixel_delta_v: DVec3::ZERO,
            pixel00_loc: DVec3::ZERO,
        }
    }

    pub fn render(&mut self, world: &impl Hittable) -> PPMImage {
        self.init();

        let mut image = PPMImage::new(self.image_width, self.image_height);

        for y in 0..image.height {
            info!("Scanlines remaining: {}", image.height - y);
            for x in 0..image.width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.spp {
                    let ray = self.get_ray(x, y);
                    pixel_color = pixel_color + Self::ray_color(&ray, self.max_depth, world);
                }
                image.set_pixel(x, y, &(pixel_color * self.pixel_samples_scale));
            }
        }
        image
    }

    fn init(&mut self) {
        let image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        self.image_height = if image_height < 1 { 1 } else { image_height };

        let viewport_height = 2.0;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);
        let focal_length = 1.0;

        self.center = DVec3::new(0.0, 0.0, 0.0);
        let viewport_u = DVec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = DVec3::new(0.0, -viewport_height, 0.0);

        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        let viewport_upper_left =
            self.center - DVec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
    }

    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let offset = self.sample_square();
        let ray_dir = (self.pixel00_loc
            + (x as f64 + offset.x) * self.pixel_delta_u
            + (y as f64 + offset.y) * self.pixel_delta_v)
            - self.center;
        Ray::new(self.center, ray_dir)
    }

    fn sample_square(&self) -> DVec3 {
        DVec3::new(random_f64() - 0.5, random_f64() - 0.5, 0.0)
    }

    fn ray_color(r: &Ray, depth: usize, world: &impl Hittable) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        if let Some(hit) = world.hit(r, &Interval::new(0.001, f64::INFINITY)) {
            if let Some(ref material) = hit.material {
                if let Some((attenuation, scattered)) = material.scatter(r, &hit) {
                    return attenuation * Self::ray_color(&scattered, depth - 1, world);
                }
            }
            return Color::new(0.0, 0.0, 0.0);
        }

        let unit_dir = r.dir.normalize();
        let a = 0.5 * (unit_dir.y + 1.0);
        let color = (1.0 - a) * DVec3::ONE + a * DVec3::new(0.5, 0.7, 1.0);
        Color::from_vec3(color)
    }
}
