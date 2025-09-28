use std::fs::File;
use std::io::Write;
use crate::math::Vec3;

pub struct FrameBuffer{ pub w:u32, pub h:u32, pub data:Vec<Vec3> }

impl FrameBuffer{
    pub fn new(w:u32,h:u32)->Self{ Self{ w,h, data: vec![Vec3::zero(); (w*h) as usize ] } }
    fn idx(&self,x:u32,y:u32)->usize{ (y*self.w + x) as usize }
    pub fn set(&mut self, x:u32, y:u32, c:Vec3) {
        let i = self.idx(x, y);
        self.data[i] = c;
        }


    pub fn as_rgba8(&self) -> Vec<u8> {
        let mut out = vec![0u8; (self.w*self.h*4) as usize];
        for (i, c) in self.data.iter().enumerate() {
            let r = (c.x.clamp(0.0,1.0)*255.0) as u8;
            let g = (c.y.clamp(0.0,1.0)*255.0) as u8;
            let b = (c.z.clamp(0.0,1.0)*255.0) as u8;
            let j = i*4;
            out[j] = r; out[j+1] = g; out[j+2] = b; out[j+3] = 255;
        }
        out
    }

    pub fn write_ppm(&self, path:&str) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        writeln!(f, "P3\n{} {}\n255", self.w, self.h)?;
        for c in &self.data {
            let r = (c.x.clamp(0.0,1.0)*255.0) as u32;
            let g = (c.y.clamp(0.0,1.0)*255.0) as u32;
            let b = (c.z.clamp(0.0,1.0)*255.0) as u32;
            writeln!(f, "{} {} {}", r, g, b)?;
        }
        Ok(())
    }
}
