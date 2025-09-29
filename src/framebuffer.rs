use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color: Image,
    bg: Color,
    cur: Color,
}

impl Framebuffer {
    pub fn new(w: u32, h: u32) -> Self {
        let color = Image::gen_image_color(w as i32, h as i32, Color::BLACK);
        Self { width: w, height: h, color, bg: Color::BLACK, cur: Color::WHITE }
    }

    pub fn clear(&mut self) {
        self.color = Image::gen_image_color(self.width as i32, self.height as i32, self.bg);
    }

    pub fn set_current_color(&mut self, c: Color) { self.cur = c; }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color.draw_pixel(x as i32, y as i32, self.cur);
        }
    }

    pub fn save_png(&self, path: &str) {
        // ignora error si no puede escribir
        let _ = self.color.export_image(path);
    }

    pub fn blit(&self, rl: &mut RaylibHandle, th: &RaylibThread) {
        if let Ok(tex) = rl.load_texture_from_image(th, &self.color) {
            let mut d = rl.begin_drawing(th);
            d.clear_background(Color::BLACK);
            d.draw_texture(&tex, 0, 0, Color::WHITE);
        }
    }
}
