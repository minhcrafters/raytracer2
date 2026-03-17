use std::sync::Arc;

use crate::{
    hittable::{HitRecord, Hittable, HittableList},
    ray::{Ray, aabb::Aabb, interval::Interval},
};

pub struct BvhNode {
    pub left: Arc<dyn Hittable>,
    pub right: Arc<dyn Hittable>,
    pub bbox: Aabb,
}

impl BvhNode {
    pub fn new(objects: Vec<Arc<dyn Hittable>>) -> Self {
        let mut items: Vec<(Arc<dyn Hittable>, Aabb)> = objects
            .into_iter()
            .map(|obj| {
                let bbox = obj.bounding_box();
                (obj, bbox)
            })
            .collect();

        Self::build(&mut items)
    }

    fn build(items: &mut [(Arc<dyn Hittable>, Aabb)]) -> Self {
        let mut bbox = Aabb::default();
        for (_, item_bbox) in items.iter() {
            bbox = Aabb::from_aabbs(&bbox, item_bbox);
        }

        let mut axis = 0;
        let x_size = bbox.x.size();
        let y_size = bbox.y.size();
        let z_size = bbox.z.size();

        if y_size > x_size && y_size > z_size {
            axis = 1;
        } else if z_size > x_size && z_size > y_size {
            axis = 2;
        }

        let object_span = items.len();

        let (left, right) = if object_span == 1 {
            (items[0].0.clone(), items[0].0.clone())
        } else if object_span == 2 {
            if Self::box_compare(&items[0], &items[1], axis) {
                (items[0].0.clone(), items[1].0.clone())
            } else {
                (items[1].0.clone(), items[0].0.clone())
            }
        } else {
            items.sort_by(|a, b| {
                if Self::box_compare(a, b, axis) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            let mid = object_span / 2;
            let (left_items, right_items) = items.split_at_mut(mid);

            let left = Arc::new(BvhNode::build(left_items)) as Arc<dyn Hittable>;
            let right = Arc::new(BvhNode::build(right_items)) as Arc<dyn Hittable>;

            (left, right)
        };

        let node_bbox = Aabb::from_aabbs(&left.bounding_box(), &right.bounding_box());

        Self {
            left,
            right,
            bbox: node_bbox,
        }
    }

    pub fn from_list(list: &HittableList) -> Self {
        Self::new(list.objects.clone())
    }

    fn box_compare(
        a: &(Arc<dyn Hittable>, Aabb),
        b: &(Arc<dyn Hittable>, Aabb),
        axis_index: usize,
    ) -> bool {
        a.1.axis(axis_index).min < b.1.axis(axis_index).min
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
