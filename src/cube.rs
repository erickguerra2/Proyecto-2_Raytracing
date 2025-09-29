use raylib::prelude::*;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

/// Cubo axis-aligned (AABB)
pub struct Cube {
    pub min: Vector3,
    pub max: Vector3,
    pub mat: Material,
    pub uv_scale: f32, // <-- cuántas veces repetir la textura (uniforme)
}

impl Cube {
    pub fn from_center_size(center: Vector3, size: Vector3, mat: Material) -> Self {
        let half = size * 0.5;
        Self { min: center - half, max: center + half, mat, uv_scale: 1.0 }
    }

    /// helper opcional para setear tiling
    pub fn with_tiling(mut self, uv_scale: f32) -> Self {
        self.uv_scale = uv_scale.max(0.001);
        self
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ro: &Vector3, rd: &Vector3) -> Intersect {
        // método de slabs
        let inv = Vector3::new(1.0/rd.x, 1.0/rd.y, 1.0/rd.z);

        let mut t1 = (self.min.x - ro.x) * inv.x;
        let mut t2 = (self.max.x - ro.x) * inv.x;
        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        t1 = (self.min.y - ro.y) * inv.y; t2 = (self.max.y - ro.y) * inv.y;
        tmin = tmin.max(t1.min(t2)); tmax = tmax.min(t1.max(t2));

        t1 = (self.min.z - ro.z) * inv.z; t2 = (self.max.z - ro.z) * inv.z;
        tmin = tmin.max(t1.min(t2)); tmax = tmax.min(t1.max(t2));

        let t = if tmax >= tmin.max(0.0) { tmin.max(0.0) } else { f32::INFINITY };
        if !t.is_finite() { return Intersect::empty(); }

        let p = *ro + *rd * t;

        // normal + uv por cara
        let eps = 1e-3;
        let mut n = Vector3::zero();
        let mut uv = (0.0, 0.0);
        // X faces
        if (p.x - self.min.x).abs() < eps { 
            n = Vector3::new(-1.0, 0.0, 0.0); 
            uv = ((p.z - self.min.z)/(self.max.z-self.min.z), (p.y - self.min.y)/(self.max.y-self.min.y)); 
        } else if (p.x - self.max.x).abs() < eps { 
            n = Vector3::new( 1.0, 0.0, 0.0); 
            uv = ((p.z - self.min.z)/(self.max.z-self.min.z), (p.y - self.min.y)/(self.max.y-self.min.y)); 
        }
        // Y faces
        else if (p.y - self.min.y).abs() < eps { 
            n = Vector3::new(0.0,-1.0, 0.0); 
            uv = ((p.x - self.min.x)/(self.max.x-self.min.x), (p.z - self.min.z)/(self.max.z-self.min.z)); 
        } else if (p.y - self.max.y).abs() < eps { 
            n = Vector3::new(0.0, 1.0, 0.0); 
            uv = ((p.x - self.min.x)/(self.max.x-self.min.x), (p.z - self.min.z)/(self.max.z-self.min.z)); 
        }
        // Z faces
        else if (p.z - self.min.z).abs() < eps { 
            n = Vector3::new(0.0, 0.0,-1.0); 
            uv = ((p.x - self.min.x)/(self.max.x-self.min.x), (p.y - self.min.y)/(self.max.y-self.min.y)); 
        } else { 
            n = Vector3::new(0.0, 0.0, 1.0); 
            uv = ((p.x - self.min.x)/(self.max.x-self.min.x), (p.y - self.min.y)/(self.max.y-self.min.y)); 
        }

        // aplica tiling (repetición)
        let uv = (uv.0 * self.uv_scale, uv.1 * self.uv_scale);

        Intersect::new(p, n, t, self.mat, uv)
    }
}
