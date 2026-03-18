use std::f64::EPSILON;

use glam::DVec3;
use std::f64::consts::PI;

pub fn random_f64() -> f64 {
    rand::random::<f64>()
}

pub fn random_f64_range(min: f64, max: f64) -> f64 {
    min + (max - min) * random_f64()
}

pub fn random_vec3() -> DVec3 {
    DVec3::new(random_f64(), random_f64(), random_f64())
}

pub fn random_vec3_range(min: f64, max: f64) -> DVec3 {
    DVec3::new(
        random_f64_range(min, max),
        random_f64_range(min, max),
        random_f64_range(min, max),
    )
}

pub fn random_unit_vec3() -> DVec3 {
    loop {
        let p = random_vec3_range(-1.0, 1.0);
        if p.length_squared() <= 1.0 && p.length_squared() > EPSILON {
            return p.normalize();
        }
    }
}

pub fn random_on_hemisphere(normal: DVec3) -> DVec3 {
    let in_unit_sphere = random_unit_vec3();
    if in_unit_sphere.dot(normal) > 0.0 {
        in_unit_sphere
    } else {
        -in_unit_sphere
    }
}

pub fn random_in_unit_disk() -> DVec3 {
    loop {
        let p = DVec3::new(
            random_f64_range(-1.0, 1.0),
            random_f64_range(-1.0, 1.0),
            0.0,
        );

        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

fn hable_operator(x: f64) -> f64 {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;
    ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f
}

pub fn tonemap(linear: f64) -> f64 {
    if linear > 0.0 {
        let exposure_bias = 2.0;
        let curr = hable_operator(linear * exposure_bias);
        let white_scale = 1.0 / hable_operator(11.2);
        let color = curr * white_scale;
        color.clamp(0.0, 1.0).powf(1.0 / 2.2)
    } else {
        0.0
    }
}

pub fn near_zero(vec: DVec3) -> bool {
    vec.x.abs() < EPSILON && vec.y.abs() < EPSILON && vec.z.abs() < EPSILON
}

pub fn random_cosine_direction() -> DVec3 {
    let r1 = random_f64();
    let r2 = random_f64();

    let phi = 2.0 * PI * r1;
    let x = f64::cos(phi) * f64::sqrt(r2);
    let y = f64::sin(phi) * f64::sqrt(r2);
    let z = f64::sqrt(1.0 - r2);

    DVec3::new(x, y, z)
}

pub fn random_to_sphere(radius: f64, distance_squared: f64) -> DVec3 {
    let r1 = random_f64();
    let r2 = random_f64();
    let z = 1.0 + r2 * ((1.0 - radius * radius / distance_squared).max(0.0).sqrt() - 1.0);

    let phi = 2.0 * std::f64::consts::PI * r1;
    let x = phi.cos() * (1.0 - z * z).max(0.0).sqrt();
    let y = phi.sin() * (1.0 - z * z).max(0.0).sqrt();

    DVec3::new(x, y, z)
}

pub struct OrthonormalBasis {
    axis: [DVec3; 3],
}

impl OrthonormalBasis {
    pub fn build_from_w(w: DVec3) -> Self {
        let w = w.normalize();
        let a = if w.x.abs() > 0.9 {
            DVec3::new(0.0, 1.0, 0.0)
        } else {
            DVec3::new(1.0, 0.0, 0.0)
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);
        Self { axis: [u, v, w] }
    }

    pub fn u(&self) -> DVec3 {
        self.axis[0]
    }
    pub fn v(&self) -> DVec3 {
        self.axis[1]
    }
    pub fn w(&self) -> DVec3 {
        self.axis[2]
    }

    pub fn local(&self, a: DVec3) -> DVec3 {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }
}
