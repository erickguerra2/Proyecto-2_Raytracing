use crate::math::{Vec3, Ray};

#[derive(Clone, Copy)]  
pub struct OrbitCamera{
    pub target: Vec3,
    pub distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub fov_y: f32,
    pub aspect: f32,
    pub eye: Vec3,
}

impl OrbitCamera{
    pub fn new(eye:Vec3,target:Vec3,fov_y:f32,aspect:f32)->Self{
        Self{target,distance:(eye-target).len(),azimuth:0.0,elevation:0.3,fov_y,aspect,eye}
    }
    pub fn update_eye(&mut self){
        let ca = self.azimuth.cos(); let sa = self.azimuth.sin();
        let ce = self.elevation.cos(); let se = self.elevation.sin();
        let x = self.distance * ce * ca;
        let y = self.distance * se;
        let z = self.distance * ce * sa;
        self.eye = self.target + Vec3::new(x,y,z);
    }
    pub fn ray(&self, u: f32, v: f32) -> Ray {
        let forward = (self.target - self.eye).normalized();
        let right = forward.cross(Vec3::new(0.0,1.0,0.0)).normalized();
        let up = right.cross(forward).normalized();
        let tan = (self.fov_y*0.5).tan();
        let px = (2.0*u - 1.0) * tan * self.aspect;
        let py = (1.0 - 2.0*v) * tan;
        let dir = (forward + right*px + up*py).normalized();
        Ray::new(self.eye, dir)
    }
}
