use raylib::prelude::*;
use crate::texture::TextureCPU;

/// Orden: nx, px, ny, py, nz, pz
pub struct Skybox {
    pub nx: TextureCPU, pub px: TextureCPU,
    pub ny: TextureCPU, pub py: TextureCPU,
    pub nz: TextureCPU, pub pz: TextureCPU,
}

impl Skybox {
    pub fn new(nx: TextureCPU, px: TextureCPU, ny: TextureCPU, py: TextureCPU, nz: TextureCPU, pz: TextureCPU) -> Self {
        Self{nx,px,ny,py,nz,pz}
    }

    /// Mapea dir a cara + uv (convención tipo OpenGL)
    pub fn sample(&self, dir: Vector3) -> Vector3 {
        let d = dir.normalized();
        let ax = d.x.abs(); let ay = d.y.abs(); let az = d.z.abs();
        let (u,v,face) = if ax >= ay && ax >= az {
            if d.x > 0.0 {
                // +X
                let u = -d.z/ax * 0.5 + 0.5; let v = -d.y/ax * 0.5 + 0.5; (u,v, 1)
            } else {
                // -X
                let u =  d.z/ax * 0.5 + 0.5; let v = -d.y/ax * 0.5 + 0.5; (u,v, 0)
            }
        } else if ay >= ax && ay >= az {
            if d.y > 0.0 {
                // +Y
                let u =  d.x/ay * 0.5 + 0.5; let v =  d.z/ay * 0.5 + 0.5; (u,v, 3)
            } else {
                // -Y
                let u =  d.x/ay * 0.5 + 0.5; let v = -d.z/ay * 0.5 + 0.5; (u,v, 2)
            }
        } else {
            if d.z > 0.0 {
                // +Z
                let u =  d.x/az * 0.5 + 0.5; let v = -d.y/az * 0.5 + 0.5; (u,v, 5)
            } else {
                // -Z
                let u = -d.x/az * 0.5 + 0.5; let v = -d.y/az * 0.5 + 0.5; (u,v, 4)
            }
        };

        match face {
            0 => self.nx.sample_repeat(u,v),
            1 => self.px.sample_repeat(u,v),
            2 => self.ny.sample_repeat(u,v),
            3 => self.py.sample_repeat(u,v),
            4 => self.nz.sample_repeat(u,v),
            _ => self.pz.sample_repeat(u,v),
        }
    }
}
