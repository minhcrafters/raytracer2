use crate::{
    hittable::HitRecord,
    image::Color,
    material::{Material, ScatterRecord},
    ray::Ray,
    utils::random_unit_vec3,
};

pub struct Metallic {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metallic {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metallic {
    fn scatter<'a>(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord<'a>> {
        let reflected =
            ray_in.dir.reflect(hit_record.normal).normalize() + self.fuzz * random_unit_vec3();
        let scattered_ray = Ray::new(hit_record.point, reflected, ray_in.time);

        if scattered_ray.dir.dot(hit_record.normal) > 0.0 {
            Some(ScatterRecord {
                attenuation: self.albedo,
                pdf: None,
                skip_pdf: true,
                skip_pdf_ray: scattered_ray,
            })
        } else {
            None
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
