use crate::math::Vec3;
use crate::material::Material;

/// Plano infinito (punto + normal)
#[derive(Clone)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub mat: Material,
}

/// Caja alineada a ejes (AABB) con material
#[derive(Clone)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub mat: Material,
}

impl Aabb {
    #[inline]
    pub fn from_min_max(min: Vec3, max: Vec3, mat: Material) -> Self {
        Self { min, max, mat }
    }
}
