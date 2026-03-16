use std::sync::Arc;

use crate::{
    hittable::{HitRecord, Hittable, HittableList},
    ray::{Ray, aabb::Aabb, interval::Interval},
    utils::random_f64_range,
};

pub struct BvhNode {
    pub left: Arc<dyn Hittable>,
    pub right: Arc<dyn Hittable>,
    pub bbox: Aabb,
}

impl BvhNode {
    pub fn new(mut objects: Vec<Arc<dyn Hittable>>) -> Self {
        let axis = (random_f64_range(0.0, 3.0) as usize) % 3;

        let object_span = objects.len();

        let (left, right) = if object_span == 1 {
            (objects[0].clone(), objects[0].clone())
        } else if object_span == 2 {
            if Self::box_compare(&objects[0], &objects[1], axis) {
                (objects[0].clone(), objects[1].clone())
            } else {
                (objects[1].clone(), objects[0].clone())
            }
        } else {
            objects.sort_by(|a, b| {
                if Self::box_compare(a, b, axis) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            let mid = object_span / 2;
            let right_objects = objects.split_off(mid);
            let left_objects = objects;

            let left = Arc::new(BvhNode::new(left_objects)) as Arc<dyn Hittable>;
            let right = Arc::new(BvhNode::new(right_objects)) as Arc<dyn Hittable>;

            (left, right)
        };

        let bbox = Aabb::from_aabbs(&left.bounding_box(), &right.bounding_box());

        Self { left, right, bbox }
    }

    pub fn from_list(list: &HittableList) -> Self {
        Self::new(list.objects.clone())
    }

    fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis_index: usize) -> bool {
        a.bounding_box().axis(axis_index).min < b.bounding_box().axis(axis_index).min
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        if !self.bbox.hit(r, *interval) {
            return None;
        }

        let hit_left = self.left.hit(r, interval);
        let hit_right;

        let right_interval = if let Some(ref rec) = hit_left {
            Interval::new(interval.min, rec.t)
        } else {
            *interval
        };

        hit_right = self.right.hit(r, &right_interval);

        if hit_right.is_some() {
            hit_right
        } else {
            hit_left
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
