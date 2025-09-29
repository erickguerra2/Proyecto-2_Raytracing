use raylib::prelude::*;
use crate::material::Material;

#[derive(Clone, Copy)]
pub struct Intersect {
    pub point: Vector3,
    pub normal: Vector3,
    pub t: f32,
    pub hit: bool,
    pub mat: Material,
    pub uv: (f32, f32),   // para texturizar
}

impl Intersect {
    pub fn new(point: Vector3, normal: Vector3, t: f32, mat: Material, uv:(f32,f32)) -> Self {
        Self { point, normal, t, hit: true, mat, uv }
    }
    pub fn empty() -> Self {
        Self { point: Vector3::zero(), normal: Vector3::zero(), t: 0.0, hit: false, mat: Material::black(), uv:(0.0,0.0) }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ro: &Vector3, rd: &Vector3) -> Intersect;
}

pub const ORIGIN_BIAS: f32 = 1e-4;

pub fn offset_origin(p: &Vector3, n: &Vector3, dir: &Vector3) -> Vector3 {
    let off = *n * ORIGIN_BIAS;
    if dir.dot(*n) < 0.0 { *p - off } else { *p + off }
}

pub fn reflect(i: &Vector3, n: &Vector3) -> Vector3 { *i - *n * 2.0 * i.dot(*n) }

pub fn refract(i: &Vector3, n: &Vector3, ior: f32) -> Option<Vector3> {
    let mut cosi = i.dot(*n).clamp(-1.0, 1.0);
    let mut etai = 1.0;
    let mut etat = ior;
    let mut nn = *n;
    if cosi > 0.0 { std::mem::swap(&mut etai, &mut etat); nn = -nn; } else { cosi = -cosi; }
    let eta = etai / etat;
    let k = 1.0 - eta*eta * (1.0 - cosi*cosi);
    if k < 0.0 { None } else { Some(*i * eta + nn * (eta * cosi - k.sqrt())) }
}
