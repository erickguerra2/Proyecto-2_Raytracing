use raylib::prelude::*;
pub struct Light { pub pos: Vector3, pub color: Vector3, pub intensity: f32 }
impl Light { pub fn new(pos: Vector3, color: Vector3, intensity: f32) -> Self { Self{pos,color,intensity} } }
