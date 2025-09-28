use crate::math::Vec3;
use crate::shapes::{Aabb, Plane};
use crate::material::CubeTex;

pub enum Light {
    Directional { dir: Vec3, intensity: Vec3 },
    Ambient(Vec3),
}

pub struct Scene {
    pub boxes_: Vec<Aabb>,
    pub planes: Vec<Plane>,
    pub lights: Vec<Light>,
    pub skybox: Option<CubeTex>,
}

impl Scene {
    pub fn new() -> Self {
        Self { boxes_: vec![], planes: vec![], lights: vec![], skybox: None }
    }
    pub fn add_box(&mut self, b: Aabb) { self.boxes_.push(b); }
    pub fn add_plane(&mut self, p: Plane) { self.planes.push(p); }
    pub fn set_skybox(&mut self, sky: CubeTex) { self.skybox = Some(sky); }
}
