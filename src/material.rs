use raylib::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub diffuse: Vector3,         // albedo base (tinte)
    pub specular_exp: f32,        // exponente especular (Phong)
    pub albedo: [f32; 4],         // [kd, ks, kr, kt] difuso, especular, reflectividad, transparencia
    pub ior: f32,                 // índice de refracción (agua≈1.33, vidrio≈1.5)
}

impl Material {
    pub fn new(diffuse: Vector3, specular_exp: f32, albedo: [f32;4], ior: f32) -> Self {
        Self { diffuse, specular_exp, albedo, ior }
    }
    pub fn black() -> Self {
        Self { diffuse: Vector3::zero(), specular_exp: 1.0, albedo: [0.0;4], ior: 1.0 }
    }
}

pub fn v3_to_color(v: Vector3) -> Color {
    Color::new(
        (v.x.clamp(0.0,1.0) * 255.0) as u8,
        (v.y.clamp(0.0,1.0) * 255.0) as u8,
        (v.z.clamp(0.0,1.0) * 255.0) as u8,
        255
    )
}
