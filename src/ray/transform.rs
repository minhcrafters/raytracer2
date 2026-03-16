use glam::{DMat4, DQuat, DVec3};

#[derive(Clone, Copy)]
pub struct Transform {
    pub m: DMat4,
    pub inv: DMat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            m: DMat4::IDENTITY,
            inv: DMat4::IDENTITY,
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn translate(mut self, translation: DVec3) -> Self {
        let t = DMat4::from_translation(translation);
        self.m = t * self.m;
        self.inv = self.m.inverse();
        self
    }

    pub fn rotate(mut self, rotation: DQuat) -> Self {
        let r = DMat4::from_quat(rotation);
        self.m = r * self.m;
        self.inv = self.m.inverse();
        self
    }

    pub fn scale(mut self, scale: DVec3) -> Self {
        let s = DMat4::from_scale(scale);
        self.m = s * self.m;
        self.inv = self.m.inverse();
        self
    }

    pub fn transform_point(&self, p: DVec3) -> DVec3 {
        self.m.transform_point3(p)
    }

    pub fn transform_vector(&self, v: DVec3) -> DVec3 {
        self.m.transform_vector3(v)
    }

    pub fn transform_normal(&self, n: DVec3) -> DVec3 {
        // Normal matrix is the transpose of the inverse
        let n_transformed = self.inv.transpose().transform_vector3(n);
        n_transformed.normalize()
    }

    pub fn inverse_transform_point(&self, p: DVec3) -> DVec3 {
        self.inv.transform_point3(p)
    }

    pub fn inverse_transform_vector(&self, v: DVec3) -> DVec3 {
        self.inv.transform_vector3(v)
    }
}
