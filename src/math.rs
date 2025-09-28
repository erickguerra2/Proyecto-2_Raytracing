use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl Vec3 {
    pub fn new(x:f32,y:f32,z:f32)->Self{Self{x,y,z}}
    pub fn zero()->Self{Self{ x:0.0,y:0.0,z:0.0 }}
    pub fn one()->Self{Self{ x:1.0,y:1.0,z:1.0 }}
    pub fn splat(v:f32)->Self{Self{ x:v,y:v,z:v }}
    pub fn dot(self, o:Self)->f32{ self.x*o.x + self.y*o.y + self.z*o.z }
    pub fn cross(self, o:Self)->Self{
        Self::new(self.y*o.z - self.z*o.y, self.z*o.x - self.x*o.z, self.x*o.y - self.y*o.x)
    }
    pub fn len(self)->f32{ self.dot(self).sqrt() }
    pub fn normalized(self)->Self{ let l=self.len(); if l>0.0 { self/l } else { self } }
    pub fn reflect(self, n:Self)->Self{ self - n * (2.0*self.dot(n)) }
    pub fn gamma_correct(self)->Self{
        Self::new(self.x.powf(1.0/2.2), self.y.powf(1.0/2.2), self.z.powf(1.0/2.2))
    }
}

impl Add for Vec3{type Output=Self;fn add(self,o:Self)->Self{Self::new(self.x+o.x,self.y+o.y,self.z+o.z)}}
impl Sub for Vec3{type Output=Self;fn sub(self,o:Self)->Self{Self::new(self.x-o.x,self.y-o.y,self.z-o.z)}}
impl Mul<f32> for Vec3{type Output=Self;fn mul(self,s:f32)->Self{Self::new(self.x*s,self.y*s,self.z*s)}}
impl Mul<Vec3> for Vec3{type Output=Self;fn mul(self,o:Self)->Self{Self::new(self.x*o.x,self.y*o.y,self.z*o.z)}}
impl Div<f32> for Vec3{type Output=Self;fn div(self,s:f32)->Self{Self::new(self.x/s,self.y/s,self.z/s)}}
impl Neg for Vec3{type Output=Self;fn neg(self)->Self{Self::new(-self.x,-self.y,-self.z)}}

#[derive(Clone, Copy, Debug)]
pub struct Ray { pub o:Vec3, pub d:Vec3 }
impl Ray { pub fn new(o:Vec3,d:Vec3)->Self{ Self{o,d:d.normalized()} } }

pub fn clamp01(x:f32)->f32{ x.max(0.0).min(1.0) }

impl Vec3 {
    // n: normal unitaria. ior = índice relativo (eta = n1/n2).
    // Devuelve None si hay reflexión interna total (TIR).
    pub fn refract(self, n: Self, ior: f32) -> Option<Self> {
        let mut cosi = (-self).dot(n).clamp(-1.0, 1.0);
        let (etai, etat, nn) = if cosi < 0.0 {
            cosi = -cosi;
            (ior, 1.0, -n)
        } else {
            (1.0, ior, n)
        };
        let eta = etai / etat;
        let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
        if k < 0.0 { None } else { Some(self * eta + nn * (eta * cosi - k.sqrt())) }
    }
}
