use std::f64::EPSILON;

use glam::DVec3;

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
    while true {
        let p = random_vec3_range(-1.0, 1.0);
        if p.length_squared() <= 1.0 && p.length_squared() > EPSILON {
            return p.normalize();
        }
    }
    DVec3::ZERO
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
    while true {
        let p = DVec3::new(
            random_f64_range(-1.0, 1.0),
            random_f64_range(-1.0, 1.0),
            0.0,
        );

        if p.length_squared() < 1.0 {
            return p;
        }
    }
    DVec3::ZERO
}

pub fn linear_to_gamma(linear: f64) -> f64 {
    if linear > 0.0 {
        linear.powf(1.0 / 2.2)
    } else {
        0.0
    }
}

pub fn near_zero(vec: DVec3) -> bool {
    vec.x.abs() < EPSILON && vec.y.abs() < EPSILON && vec.z.abs() < EPSILON
}
