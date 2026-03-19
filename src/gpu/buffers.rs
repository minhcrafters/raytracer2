use bytemuck::{Pod, Zeroable};

pub const MAT_LAMBERTIAN: u32 = 0;
pub const MAT_METALLIC: u32 = 1;
pub const MAT_DIELECTRIC: u32 = 2;
pub const MAT_DIFFUSE_LIGHT: u32 = 3;
pub const MAT_SPECULAR: u32 = 4;

pub const PRIM_SPHERE: u32 = 0;
pub const PRIM_QUAD: u32 = 1;
pub const PRIM_TRIANGLE: u32 = 2;

pub const BG_COLOR: u32 = 0;
pub const BG_HDRI: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuCamera {
    pub inv_view: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
    pub center: [f32; 3],
    pub image_width: u32,
    pub defocus_disk_u: [f32; 3],
    pub image_height: u32,
    pub defocus_disk_v: [f32; 3],
    pub max_depth: u32,
    pub defocus_angle: f32,
    pub current_sample: u32,
    pub bg_type: u32,
    pub _pad0: u32, // ensures bg_color is 16-byte aligned
    pub bg_color: [f32; 3],
    pub hdri_width: u32,
    pub hdri_height: u32,
    pub _pad1: u32,
    pub _pad2: u32,
    pub _pad3: u32, // final padding for 16-byte alignment
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuMaterial {
    pub mat_type: u32,
    pub _pad0: [u32; 3],
    pub albedo: [f32; 3],
    pub ior: f32,
    pub emit: [f32; 3],
    pub fuzz: f32,
    pub shininess: f32,
    // for texture-based materials
    pub has_texture: u32,
    pub tex_width: u32,
    pub tex_offset: u32, // offset into texture data buffer
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuTransform {
    pub m: [[f32; 4]; 4],
    pub inv: [[f32; 4]; 4],
}

impl Default for GpuTransform {
    fn default() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            inv: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuQuad {
    pub q: [f32; 3],
    pub d: f32,
    pub u: [f32; 3],
    pub _pad0: f32,
    pub v: [f32; 3],
    pub _pad1: f32,
    pub normal: [f32; 3],
    pub _pad2: f32,
    pub w: [f32; 3],
    pub _pad3: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuTriangle {
    pub v0: [f32; 3],
    pub _pad0: f32,
    pub v1: [f32; 3],
    pub _pad1: f32,
    pub v2: [f32; 3],
    pub _pad2: f32,
    pub n0: [f32; 3],
    pub _pad3: f32,
    pub n1: [f32; 3],
    pub _pad4: f32,
    pub n2: [f32; 3],
    pub _pad5: f32,
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub uv2: [f32; 2],
    pub has_normals: u32,
    pub has_uvs: u32,
    pub _pad6: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuPrimitive {
    pub prim_type: u32,
    pub data_index: u32,
    pub material_index: u32,
    pub transform_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuBvhNode {
    pub bbox_min: [f32; 3],
    pub left_or_prim: u32, // if leaf: start_prim; if interior: left child index
    pub bbox_max: [f32; 3],
    pub right_or_count: u32, // if leaf: prim_count; if interior: right child index
    pub is_leaf: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuLight {
    pub prim_index: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuHdriPixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuTexPixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub _pad: f32,
}
