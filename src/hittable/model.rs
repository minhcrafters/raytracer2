use std::sync::Arc;

use asset_importer::{Importer, postprocess::PostProcessSteps};
use glam::DVec3;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::{Ray, aabb::Aabb, interval::Interval, transform::Transform},
};

pub struct TriMesh {
    pub vertices: Vec<DVec3>,
    pub normals: Vec<DVec3>,
    pub indices: Vec<u32>,
    pub material: Arc<dyn Material>,
    pub smooth_normals: bool,
    pub transform: Transform,
    pub bvh_root: TriBvhNode,
}

pub struct TriBvhNode {
    pub bbox: Aabb,
    pub left: Option<Box<TriBvhNode>>,
    pub right: Option<Box<TriBvhNode>>,
    pub start_tri: usize,
    pub tri_count: usize,
}

#[derive(Clone, Copy)]
struct Face {
    i0: u32,
    i1: u32,
    i2: u32,
    bbox: Aabb,
    centroid: DVec3,
}

impl TriBvhNode {
    fn build(faces: &mut [Face], start_idx: usize) -> Self {
        let mut bbox = Aabb::default();
        for face in faces.iter() {
            bbox = Aabb::from_aabbs(&bbox, &face.bbox);
        }

        let num_tris = faces.len();
        if num_tris <= 4 {
            return Self {
                bbox,
                left: None,
                right: None,
                start_tri: start_idx,
                tri_count: num_tris,
            };
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

        faces.sort_by(|a, b| {
            let ac = a.centroid[axis];
            let bc = b.centroid[axis];
            ac.partial_cmp(&bc).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = num_tris / 2;
        let (left_faces, right_faces) = faces.split_at_mut(mid);

        let left = Box::new(Self::build(left_faces, start_idx));
        let right = Box::new(Self::build(right_faces, start_idx + mid));

        Self {
            bbox,
            left: Some(left),
            right: Some(right),
            start_tri: 0,
            tri_count: 0,
        }
    }

    fn hit(&self, mesh: &TriMesh, ray_obj: &Ray, interval: &Interval) -> Option<HitRecord> {
        if !self.bbox.hit(ray_obj, *interval) {
            return None;
        }

        if self.tri_count > 0 {
            let mut hit_record = None;
            let mut closest_so_far = interval.max;

            for i in 0..self.tri_count {
                let idx = (self.start_tri + i) * 3;
                let i0 = mesh.indices[idx] as usize;
                let i1 = mesh.indices[idx + 1] as usize;
                let i2 = mesh.indices[idx + 2] as usize;

                let p0 = mesh.vertices[i0];
                let p1 = mesh.vertices[i1];
                let p2 = mesh.vertices[i2];

                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let h = ray_obj.dir.cross(edge2);
                let a = edge1.dot(h);

                if a > -1e-8 && a < 1e-8 {
                    continue;
                }

                let f = 1.0 / a;
                let s = ray_obj.origin - p0;
                let u = f * s.dot(h);

                if !(0.0..=1.0).contains(&u) {
                    continue;
                }

                let q = s.cross(edge1);
                let v = f * ray_obj.dir.dot(q);

                if v < 0.0 || u + v > 1.0 {
                    continue;
                }

                let t = f * edge2.dot(q);

                if t > interval.min && t < closest_so_far {
                    closest_so_far = t;
                    let intersection_obj = ray_obj.at(t);

                    let normal_obj = if mesh.smooth_normals && !mesh.normals.is_empty() {
                        let n0 = mesh.normals[i0];
                        let n1 = mesh.normals[i1];
                        let n2 = mesh.normals[i2];
                        let w = 1.0 - u - v;
                        (w * n0 + u * n1 + v * n2).normalize()
                    } else {
                        edge1.cross(edge2).normalize()
                    };

                    let rec = HitRecord {
                        point: intersection_obj,
                        normal: normal_obj,
                        material: Some(mesh.material.clone()),
                        t,
                        u,
                        v,
                        front_face: false,
                    };

                    hit_record = Some(rec);
                }
            }
            return hit_record;
        }

        let hit_left = self.left.as_ref().unwrap().hit(mesh, ray_obj, interval);

        let right_interval = if let Some(ref rec) = hit_left {
            Interval::new(interval.min, rec.t)
        } else {
            *interval
        };

        let hit_right = self
            .right
            .as_ref()
            .unwrap()
            .hit(mesh, ray_obj, &right_interval);

        if hit_right.is_some() {
            hit_right
        } else {
            hit_left
        }
    }
}

impl TriMesh {
    pub fn load_model(
        path: &str,
        material: Arc<dyn Material>,
        smooth_normals: bool,
    ) -> Result<Vec<TriMesh>, Box<dyn std::error::Error>> {
        let mut meshes = Vec::new();

        let scene = Importer::new()
            .read_file(path)
            .with_post_process(
                PostProcessSteps::TRIANGULATE
                    | PostProcessSteps::FLIP_UVS
                    | PostProcessSteps::JOIN_IDENTICAL_VERTICES
                    | PostProcessSteps::GEN_SMOOTH_NORMALS,
            )
            .import()?;

        for mesh in scene.meshes() {
            let vertices: Vec<DVec3> = mesh
                .vertices_iter()
                .map(|v| DVec3::new(v.x as f64, v.y as f64, v.z as f64))
                .collect();

            let normals: Vec<DVec3> = mesh
                .normals_iter()
                .map(|n| DVec3::new(n.x as f64, n.y as f64, n.z as f64))
                .collect();

            let mut indices = Vec::new();
            for face in mesh.faces() {
                let face_indices = face.indices();
                if face_indices.len() == 3 {
                    indices.extend_from_slice(face_indices);
                }
            }

            let mut faces = Vec::with_capacity(indices.len() / 3);
            for i in (0..indices.len()).step_by(3) {
                let i0 = indices[i];
                let i1 = indices[i + 1];
                let i2 = indices[i + 2];
                let p0 = vertices[i0 as usize];
                let p1 = vertices[i1 as usize];
                let p2 = vertices[i2 as usize];

                let min_p = p0.min(p1).min(p2);
                let max_p = p0.max(p1).max(p2);
                let mut bbox = Aabb::from_points(min_p, max_p);

                let delta = 0.0001;
                if bbox.x.size() < delta {
                    bbox.x = Interval::new(bbox.x.min - delta / 2.0, bbox.x.max + delta / 2.0);
                }
                if bbox.y.size() < delta {
                    bbox.y = Interval::new(bbox.y.min - delta / 2.0, bbox.y.max + delta / 2.0);
                }
                if bbox.z.size() < delta {
                    bbox.z = Interval::new(bbox.z.min - delta / 2.0, bbox.z.max + delta / 2.0);
                }

                faces.push(Face {
                    i0,
                    i1,
                    i2,
                    bbox,
                    centroid: (p0 + p1 + p2) / 3.0,
                });
            }

            let bvh_root = TriBvhNode::build(&mut faces, 0);

            let mut new_indices = Vec::with_capacity(indices.len());
            for face in faces {
                new_indices.push(face.i0);
                new_indices.push(face.i1);
                new_indices.push(face.i2);
            }

            meshes.push(TriMesh {
                vertices,
                normals,
                indices: new_indices,
                material: material.clone(),
                smooth_normals,
                transform: Transform::default(),
                bvh_root,
            });
        }

        Ok(meshes)
    }
}

impl Hittable for TriMesh {
    fn hit(&self, r: &Ray, interval: &Interval) -> Option<HitRecord> {
        let ray_obj = Ray::new(
            self.transform.inverse_transform_point(r.origin),
            self.transform.inverse_transform_vector(r.dir),
            r.time,
        );

        if let Some(mut rec) = self.bvh_root.hit(self, &ray_obj, interval) {
            let normal_world = self.transform.transform_normal(rec.normal);
            rec.point = self.transform.transform_point(rec.point);
            rec.normal = normal_world;
            rec.set_face_normal(r, normal_world);
            Some(rec)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.bvh_root.bbox.transform(&self.transform)
    }
}
