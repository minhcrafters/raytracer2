use glam::{DMat4, DVec3};

use crate::{
    background::Background,
    hittable::Hittable,
    image::{Color, PPMImage},
    pdf::{HittablePdf, MixturePdf, Pdf},
    ray::{Ray, interval::Interval},
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

    pub background: Background,

    image_height: usize,
    center: DVec3,
    pixel_samples_scale: f64,

    // MVP transformation matrices
    pub view_mat: DMat4,
    pub inv_view_mat: DMat4,
    pub proj_mat: DMat4,
    pub inv_proj_mat: DMat4,

    defocus_disk_u: DVec3,
    defocus_disk_v: DVec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: usize, spp: usize, max_depth: usize) -> Self {
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
            background: Background::Color(Color::new(0.5, 0.7, 1.0)),
            image_height: 0,
            pixel_samples_scale: 1.0 / (spp as f64),
            center: DVec3::ZERO,

            view_mat: DMat4::IDENTITY,
            inv_view_mat: DMat4::IDENTITY,
            proj_mat: DMat4::IDENTITY,
            inv_proj_mat: DMat4::IDENTITY,

            defocus_disk_u: DVec3::ZERO,
            defocus_disk_v: DVec3::ZERO,
        }
    }

    pub fn render(&mut self, world: &impl Hittable, lights: &impl Hittable) -> PPMImage {
        self.init();

        let mut image = PPMImage::new(self.image_width, self.image_height);

        use rayon::prelude::*;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let this = &*self;
        let lines_completed = AtomicUsize::new(0);
        use std::io::Write;

        let pixels: Vec<_> = (0..image.height)
            .into_par_iter()
            .map(|y| {
                let row_pixels: Vec<_> = (0..image.width)
                    .map(|x| {
                        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                        for _ in 0..this.spp {
                            let ray = this.get_ray(x, y);
                            pixel_color =
                                pixel_color + this.ray_color(&ray, this.max_depth, world, lights);
                        }
                        (x, y, pixel_color * this.pixel_samples_scale)
                    })
                    .collect();

                let completed = lines_completed.fetch_add(1, Ordering::Relaxed) + 1;
                print!("\rScanlines remaining: {} ", image.height - completed);
                let _ = std::io::stdout().flush();

                row_pixels
            })
            .flatten()
            .collect();

        for (x, y, color) in pixels {
            image.set_pixel(x, y, &color);
        }

        image
    }

    fn init(&mut self) {
        let image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        self.image_height = if image_height < 1 { 1 } else { image_height };
        let actual_aspect_ratio = self.image_width as f64 / self.image_height as f64;

        self.center = self.look_from;

        // View Matrix
        self.view_mat = DMat4::look_at_rh(self.look_from, self.look_at, self.vup);
        self.inv_view_mat = self.view_mat.inverse();

        // Projection Matrix
        self.proj_mat =
            DMat4::perspective_rh(self.fov.to_radians(), actual_aspect_ratio, 0.001, 10000.0);
        self.inv_proj_mat = self.proj_mat.inverse();

        // Defocus blur
        let w = (self.look_from - self.look_at).normalize();
        let u = self.vup.cross(w).normalize();
        let v = w.cross(u);

        let defocus_radius = self.focus_dist * (self.defocus_angle / 2.0).to_radians().tan();
        self.defocus_disk_u = u * defocus_radius;
        self.defocus_disk_v = v * defocus_radius;
    }

    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let offset = self.sample_square();

        // Normalized Device Coordinates (NDC)
        let s = (x as f64 + offset.x) / self.image_width as f64;
        let t = (y as f64 + offset.y) / self.image_height as f64;

        // Map s, t to [-1, 1]. In rendering usually top-left is (0,0) and +Y is up in 3D.
        let ndc_x = s * 2.0 - 1.0;
        let ndc_y = 1.0 - t * 2.0;

        // Generate target point in view space by un-projecting
        // Ray starts at origin (0,0,0) in view space, goes to projected screen plane.
        let target_view = self
            .inv_proj_mat
            .project_point3(DVec3::new(ndc_x, ndc_y, -1.0));
        let ray_dir_view = target_view.normalize();

        // Translate view direction back to world space
        let ray_dir_world = self
            .inv_view_mat
            .transform_vector3(ray_dir_view)
            .normalize();

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        Ray::new(ray_origin, ray_dir_world, crate::utils::random_f64())
    }

    fn sample_square(&self) -> DVec3 {
        DVec3::new(random_f64() - 0.5, random_f64() - 0.5, 0.0)
    }

    fn defocus_disk_sample(&self) -> DVec3 {
        let p = random_in_unit_disk();
        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    fn ray_color(
        &self,
        r: &Ray,
        depth: usize,
        world: &impl Hittable,
        lights: &impl Hittable,
    ) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        if let Some(hit) = world.hit(r, &Interval::new(0.001, f64::INFINITY)) {
            let emission = if let Some(ref material) = hit.material {
                material.emitted(r, &hit)
            } else {
                Color::new(0.0, 0.0, 0.0)
            };

            if let Some(ref material) = hit.material {
                if let Some(srec) = material.scatter(r, &hit) {
                    if srec.skip_pdf {
                        return emission
                            + srec.attenuation
                                * self.ray_color(&srec.skip_pdf_ray, depth - 1, world, lights);
                    }

                    let material_pdf = srec.pdf.unwrap();

                    let scattered;
                    let pdf_val;

                    if lights.is_empty() {
                        scattered = Ray::new(hit.point, material_pdf.generate(), r.time);
                        pdf_val = material_pdf.value(scattered.dir);
                    } else {
                        let light_ptr = HittablePdf::new(lights, hit.point);
                        let p = MixturePdf::new(&light_ptr, material_pdf.as_ref());
                        scattered = Ray::new(hit.point, p.generate(), r.time);
                        pdf_val = p.value(scattered.dir);
                    }

                    let scattering_pdf = material.scattering_pdf(r, &hit, &scattered);

                    let scatter_color = (srec.attenuation
                        * scattering_pdf
                        * self.ray_color(&scattered, depth - 1, world, lights))
                        / pdf_val;

                    // Avoid fireflies
                    let mut final_color = emission + scatter_color;
                    final_color.r = final_color.r.min(10.0);
                    final_color.g = final_color.g.min(10.0);
                    final_color.b = final_color.b.min(10.0);
                    if final_color.r.is_nan() {
                        final_color.r = 0.0;
                    }
                    if final_color.g.is_nan() {
                        final_color.g = 0.0;
                    }
                    if final_color.b.is_nan() {
                        final_color.b = 0.0;
                    }

                    return final_color;
                }
            }
            return emission;
        }

        self.background.sample(r.dir)
    }
}
