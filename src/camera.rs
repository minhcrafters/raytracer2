use std::f64::INFINITY;

use glam::DVec3;
use log::info;

use crate::{
    image::{Color, PPMImage},
    ray::{Ray, hittable::Hittable, interval::Interval},
    utils::{random_f64, random_in_unit_disk},
};

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub spp: usize,
    pub max_depth: usize,

    pub fov: f64,

    pub look_from: DVec3,
    pub look_at: DVec3,
    pub vup: DVec3,

    pub defocus_angle: f64,
    pub focus_dist: f64,

    pub background: Color,

    image_height: usize,
    center: DVec3,
    pixel_delta_u: DVec3,
    pixel_delta_v: DVec3,
    u: DVec3,
    v: DVec3,
    w: DVec3,
    pixel00_loc: DVec3,
    pixel_samples_scale: f64,

    defocus_disk_u: DVec3,
    defocus_disk_v: DVec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: usize, spp: usize, max_depth: usize) -> Self {
        let background = Color::new(0.7, 0.8, 1.0);
        Self {
            aspect_ratio,
            image_width,
            spp,
            max_depth,
            fov: 90.0,
            look_from: DVec3::new(0.0, 0.0, 5.0),
            look_at: DVec3::new(0.0, 0.0, 0.0),
            vup: DVec3::Y,
            defocus_angle: 0.0,
            focus_dist: 10.0,
            background,
            image_height: 0,
            pixel_samples_scale: 1.0 / (spp as f64),
            center: DVec3::ZERO,
            pixel_delta_u: DVec3::ZERO,
            pixel_delta_v: DVec3::ZERO,
            pixel00_loc: DVec3::ZERO,
            u: DVec3::ZERO,
            v: DVec3::ZERO,
            w: DVec3::ZERO,
            defocus_disk_u: DVec3::ZERO,
            defocus_disk_v: DVec3::ZERO,
        }
    }

    pub fn render(&mut self, world: &impl Hittable) -> PPMImage {
        self.init();

        let mut image = PPMImage::new(self.image_width, self.image_height);

        for y in 0..image.height {
            print!("\rScanlines remaining: {}", image.height - y);
            for x in 0..image.width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.spp {
                    let ray = self.get_ray(x, y);
                    pixel_color = pixel_color + self.ray_color(&ray, self.max_depth, world);
                }
                image.set_pixel(x, y, &(pixel_color * self.pixel_samples_scale));
            }
        }
        image
    }

    fn init(&mut self) {
        let image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        self.image_height = if image_height < 1 { 1 } else { image_height };

        self.center = self.look_from;

        let theta = self.fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.w = (self.look_from - self.look_at).normalize();
        self.u = self.vup.cross(self.w).normalize();
        self.v = self.w.cross(self.u);

        let viewport_u = viewport_width * self.u;
        let viewport_v = viewport_height * -self.v;

        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        let viewport_upper_left =
            self.center - (self.focus_dist * self.w) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        let defocus_radius = self.focus_dist * (self.defocus_angle / 2.0).to_radians().tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let offset = self.sample_square();
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        let ray_dir = (self.pixel00_loc
            + (x as f64 + offset.x) * self.pixel_delta_u
            + (y as f64 + offset.y) * self.pixel_delta_v)
            - ray_origin;

        Ray::new(ray_origin, ray_dir, crate::utils::random_f64())
    }

    fn sample_square(&self) -> DVec3 {
        DVec3::new(random_f64() - 0.5, random_f64() - 0.5, 0.0)
    }

    fn defocus_disk_sample(&self) -> DVec3 {
        let p = random_in_unit_disk();
        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    fn ray_color(&self, r: &Ray, depth: usize, world: &impl Hittable) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        if let Some(hit) = world.hit(r, &Interval::new(0.001, f64::INFINITY)) {
            let emission = if let Some(ref material) = hit.material {
                material.emitted(hit.u, hit.v, hit.point)
            } else {
                Color::new(0.0, 0.0, 0.0)
            };

            if let Some(ref material) = hit.material {
                if let Some((attenuation, scattered)) = material.scatter(r, &hit) {
                    return emission + attenuation * self.ray_color(&scattered, depth - 1, world);
                }
            }
            return emission;
        }

        self.background
    }
}
