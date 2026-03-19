use crate::{
    hittable::Hittable,
    utils::{OrthonormalBasis, random_cosine_direction, random_f64},
};
use glam::DVec3;
use std::f64::consts::PI;

pub trait Pdf {
    fn value(&self, direction: DVec3) -> f64;
    fn generate(&self) -> DVec3;
}

pub struct CosinePdf {
    uvw: OrthonormalBasis,
}

impl CosinePdf {
    pub fn new(w: DVec3) -> Self {
        Self {
            uvw: OrthonormalBasis::build_from_w(w),
        }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: DVec3) -> f64 {
        let cosine_theta = direction.normalize().dot(self.uvw.w());
        if cosine_theta <= 0.0 {
            0.0
        } else {
            cosine_theta / PI
        }
    }

    fn generate(&self) -> DVec3 {
        self.uvw.local(random_cosine_direction())
    }
}

pub struct HittablePdf<'a> {
    hittable: &'a dyn Hittable,
    origin: DVec3,
}

impl<'a> HittablePdf<'a> {
    pub fn new(hittable: &'a dyn Hittable, origin: DVec3) -> Self {
        Self { hittable, origin }
    }
}

impl<'a> Pdf for HittablePdf<'a> {
    fn value(&self, direction: DVec3) -> f64 {
        self.hittable.pdf_value(self.origin, direction)
    }

    fn generate(&self) -> DVec3 {
        self.hittable.random(self.origin)
    }
}

pub struct MixturePdf<'a, 'b> {
    p0: &'a dyn Pdf,
    p1: &'b dyn Pdf,
}

impl<'a, 'b> MixturePdf<'a, 'b> {
    pub fn new(p0: &'a dyn Pdf, p1: &'b dyn Pdf) -> Self {
        Self { p0, p1 }
    }
}

impl<'a, 'b> Pdf for MixturePdf<'a, 'b> {
    fn value(&self, direction: DVec3) -> f64 {
        0.5 * self.p0.value(direction) + 0.5 * self.p1.value(direction)
    }

    fn generate(&self) -> DVec3 {
        if random_f64() < 0.5 {
            self.p0.generate()
        } else {
            self.p1.generate()
        }
    }
}

pub struct SpherePdf;

impl SpherePdf {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pdf for SpherePdf {
    fn value(&self, _direction: DVec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> DVec3 {
        crate::utils::random_unit_vec3()
    }
}
