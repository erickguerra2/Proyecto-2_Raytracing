use raylib::prelude::*;

pub struct TextureCPU {
    pub w: i32,
    pub h: i32,
    pub data: Vec<Color>, // row-major
}

impl TextureCPU {
    pub fn from_image(img: &Image) -> Option<Self> {
        // get_image_data() -> ImageColors, lo convertimos a Vec<Color>
        let colors = img.get_image_data();
        let data: Vec<Color> = colors.to_vec();
        Some(Self { w: img.width(), h: img.height(), data })
    }

    pub fn sample_repeat(&self, mut u: f32, mut v: f32) -> Vector3 {
        u = u.fract(); if u < 0.0 { u += 1.0; }
        v = v.fract(); if v < 0.0 { v += 1.0; }
        let x = (u * self.w as f32).floor().clamp(0.0, (self.w - 1) as f32) as i32;
        let y = ((1.0 - v) * self.h as f32).floor().clamp(0.0, (self.h - 1) as f32) as i32;
        let idx = (y * self.w + x) as usize;
        let c = self.data[idx];
        Vector3::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
    }
}
