use crate::ray::{
    Ray,
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    interval::Interval,
    transform::Transform,
};
use std::sync::Arc;

pub struct Instance {
    pub object: Arc<dyn Hittable>,
    pub transform: Transform,
    bbox: Aabb,
}

impl Instance {
    pub fn new(object: Arc<dyn Hittable>, transform: Transform) -> Self {
        let bbox = object.bounding_box();
        let mut min = glam::DVec3::splat(f64::INFINITY);
        let mut max = glam::DVec3::splat(f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = if i == 0 { bbox.x.min } else { bbox.x.max };
                    let y = if j == 0 { bbox.y.min } else { bbox.y.max };
                    let z = if k == 0 { bbox.z.min } else { bbox.z.max };

                    let tester = glam::DVec3::new(x, y, z);
                    let transformed = transform.transform_point(tester);

                    min = min.min(transformed);
                    max = max.max(transformed);
                }
            }
        }

        let new_bbox = Aabb::from_points(min, max);

        Self {
            object,
            transform,
            bbox: new_bbox,
        }
    }
}

impl Hittable for Instance {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let origin = self.transform.inverse_transform_point(r.origin);
        let dir = self.transform.inverse_transform_vector(r.dir);
        let transformed_ray = Ray::new(origin, dir, r.time);

        if let Some(mut hit) = self.object.hit(&transformed_ray, interval) {
            hit.point = self.transform.transform_point(hit.point);
            let normal = self.transform.transform_normal(hit.normal);
            hit.set_face_normal(r, normal);
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
