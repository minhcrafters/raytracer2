use std::sync::Arc;

use crate::{
    hittable::{HitRecord, Hittable, HittableList},
    ray::{Ray, aabb::Aabb, interval::Interval},
};

const NUM_SAH_BINS: usize = 12;
const TRAVERSAL_COST: f64 = 1.0;
const INTERSECT_COST: f64 = 1.0;

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

        let object_span = items.len();

        if object_span == 1 {
            return Self {
                left: items[0].0.clone(),
                right: items[0].0.clone(),
                bbox,
            };
        }

        if object_span == 2 {
            return Self {
                left: items[0].0.clone(),
                right: items[1].0.clone(),
                bbox: Aabb::from_aabbs(&items[0].1, &items[1].1),
            };
        }

        // SAH binned evaluation across all 3 axes
        let mut best_cost = f64::INFINITY;
        let mut best_axis = 0;
        let mut best_split = 0usize;

        for axis in 0..3 {
            let axis_size = bbox.axis(axis).size();
            if axis_size < 1e-10 {
                continue;
            }

            let axis_min = bbox.axis(axis).min;

            // bin primitives
            let mut bins_count = [0usize; NUM_SAH_BINS];
            let mut bins_bbox = [Aabb::default(); NUM_SAH_BINS];

            for (_, item_bbox) in items.iter() {
                let centroid = (item_bbox.axis(axis).min + item_bbox.axis(axis).max) * 0.5;
                let bin = ((centroid - axis_min) / axis_size * NUM_SAH_BINS as f64) as usize;
                let bin = bin.min(NUM_SAH_BINS - 1);
                bins_count[bin] += 1;
                bins_bbox[bin] = Aabb::from_aabbs(&bins_bbox[bin], item_bbox);
            }

            // sweep from left to build prefix surface areas and counts
            let mut left_count = [0usize; NUM_SAH_BINS - 1];
            let mut left_area = [0.0f64; NUM_SAH_BINS - 1];
            let mut running_bbox = Aabb::default();
            let mut running_count = 0usize;

            for i in 0..(NUM_SAH_BINS - 1) {
                running_count += bins_count[i];
                running_bbox = Aabb::from_aabbs(&running_bbox, &bins_bbox[i]);
                left_count[i] = running_count;
                left_area[i] = surface_area(&running_bbox);
            }

            // sweep from right
            let mut right_count = [0usize; NUM_SAH_BINS - 1];
            let mut right_area = [0.0f64; NUM_SAH_BINS - 1];
            running_bbox = Aabb::default();
            running_count = 0;

            for i in (0..(NUM_SAH_BINS - 1)).rev() {
                running_count += bins_count[i + 1];
                running_bbox = Aabb::from_aabbs(&running_bbox, &bins_bbox[i + 1]);
                right_count[i] = running_count;
                right_area[i] = surface_area(&running_bbox);
            }

            // evaluate SAH cost at each split plane
            for i in 0..(NUM_SAH_BINS - 1) {
                if left_count[i] == 0 || right_count[i] == 0 {
                    continue;
                }
                let cost = TRAVERSAL_COST
                    + INTERSECT_COST
                        * (left_count[i] as f64 * left_area[i]
                            + right_count[i] as f64 * right_area[i])
                        / surface_area(&bbox);

                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_split = i;
                }
            }
        }

        // leaf cost: testing all primitives
        let leaf_cost = INTERSECT_COST * object_span as f64;

        // if no valid split found or SAH says a leaf is cheaper, fall back to midpoint
        if best_cost >= leaf_cost {
            // still need to split since we only support binary trees
            let axis = best_axis;
            items.sort_by(|a, b| {
                let ca = (a.1.axis(axis).min + a.1.axis(axis).max) * 0.5;
                let cb = (b.1.axis(axis).min + b.1.axis(axis).max) * 0.5;
                ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
            });
            let mid = object_span / 2;
            let (left_items, right_items) = items.split_at_mut(mid);
            let left = Arc::new(BvhNode::build(left_items)) as Arc<dyn Hittable>;
            let right = Arc::new(BvhNode::build(right_items)) as Arc<dyn Hittable>;
            let bbox = Aabb::from_aabbs(&left.bounding_box(), &right.bounding_box());
            return Self { left, right, bbox };
        }

        // partition items using the SAH-chosen split
        let axis_min = bbox.axis(best_axis).min;
        let axis_size = bbox.axis(best_axis).size();

        items.sort_by(|a, b| {
            let ca = (a.1.axis(best_axis).min + a.1.axis(best_axis).max) * 0.5;
            let cb = (b.1.axis(best_axis).min + b.1.axis(best_axis).max) * 0.5;
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        // find the partition point matching the chosen bin split
        let split_pos = axis_min + (best_split as f64 + 1.0) / NUM_SAH_BINS as f64 * axis_size;

        let mid = items
            .iter()
            .position(|(_, b)| {
                let c = (b.axis(best_axis).min + b.axis(best_axis).max) * 0.5;
                c >= split_pos
            })
            .unwrap_or(object_span / 2);

        // ensure non-empty partitions
        let mid = mid.clamp(1, object_span - 1);

        let (left_items, right_items) = items.split_at_mut(mid);

        let left = Arc::new(BvhNode::build(left_items)) as Arc<dyn Hittable>;
        let right = Arc::new(BvhNode::build(right_items)) as Arc<dyn Hittable>;

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
}

fn surface_area(bbox: &Aabb) -> f64 {
    let dx = bbox.x.size().max(0.0);
    let dy = bbox.y.size().max(0.0);
    let dz = bbox.z.size().max(0.0);
    2.0 * (dx * dy + dy * dz + dz * dx)
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        if !self.bbox.hit(r, *interval) {
            return None;
        }

        let hit_left = self.left.hit(r, interval);

        let right_interval = if let Some(ref rec) = hit_left {
            Interval::new(interval.min, rec.t)
        } else {
            *interval
        };

        let hit_right = self.right.hit(r, &right_interval);

        if hit_right.is_some() {
            hit_right
        } else {
            hit_left
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
