use std::ffi::CString;
use raylib::prelude::*;
use raylib::ffi;

use crate::math::Vec3;

#[derive(Clone)]
pub struct ImageTex {
    pub w: i32,
    pub h: i32,
    pub pixels: Vec<Color>,
    /// cuántas veces se repite la textura sobre 1.0x1.0 en coordenadas (tiling)
    pub tile: f32,
}

#[derive(Clone)]
pub struct CubeTex {
    /// Orden: +X, -X, +Y, -Y, +Z, -Z
    pub faces: [ImageTex; 6],
}

/// Carga una imagen RGBA8 desde disco y la deja lista para samplear (con tiling).
pub fn load_image_tex(path: &str, tile: f32) -> ImageTex {
    unsafe {
        let c = CString::new(path).expect("CString");
        let mut img = ffi::LoadImage(c.as_ptr());
        // Asegurar formato RGBA8
        ffi::ImageFormat(&mut img, ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32);

        let w = img.width;
        let h = img.height;
        let len = (w * h) as usize;

        // Los píxeles vienen como raylib::ffi::Color (RGBA8)
        let data = img.data as *const ffi::Color;
        let slice = std::slice::from_raw_parts(data, len);

        // Copiamos a raylib::prelude::Color (mismo layout práctico)
        let mut pixels = Vec::<Color>::with_capacity(len);
        for c in slice.iter() {
            pixels.push(Color { r: c.r, g: c.g, b: c.b, a: c.a });
        }

        ffi::UnloadImage(img);

        ImageTex { w, h, pixels, tile }
    }
}

/// Carga un cubemap a partir de las 6 caras. (px, nx, py, ny, pz, nz)
pub fn load_cubemap(px: &str, nx: &str, py: &str, ny: &str, pz: &str, nz: &str) -> CubeTex {
    CubeTex {
        faces: [
            load_image_tex(px, 1.0),
            load_image_tex(nx, 1.0),
            load_image_tex(py, 1.0),
            load_image_tex(ny, 1.0),
            load_image_tex(pz, 1.0),
            load_image_tex(nz, 1.0),
        ],
    }
}

/// Muestra la textura con wrapping/tiling (u,v en espacio continuo; se toma fract()).
pub fn sample_image(tex: &ImageTex, u: f32, v: f32) -> Vec3 {
    // Aplicar tiling y wrap (fract)
    let mut uu = (u * tex.tile).fract();
    if uu < 0.0 { uu += 1.0; }
    let mut vv = (v * tex.tile).fract();
    if vv < 0.0 { vv += 1.0; }

    // Pixel más cercano
    let x = (uu * (tex.w as f32)) as i32;
    let y = (vv * (tex.h as f32)) as i32;
    let xi = x.clamp(0, tex.w - 1) as usize;
    let yi = y.clamp(0, tex.h - 1) as usize;

    let c = tex.pixels[yi * tex.w as usize + xi];
    Vec3::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
    )
}

/// Triplanar: mezcla 3 proyecciones ortogonales en base al |normal|.
/// Usa coordenadas de mundo (p) directamente; el tiling lo controla tex.tile.
pub fn sample_triplanar(tex: &ImageTex, p: &Vec3, n: &Vec3) -> Vec3 {
    let an = Vec3::new(n.x.abs(), n.y.abs(), n.z.abs());
    let w_sum = an.x + an.y + an.z + 1e-6;
    let wx = an.x / w_sum;
    let wy = an.y / w_sum;
    let wz = an.z / w_sum;

    // mapeos:
    // eje X → (z, y)   | eje Y → (x, z)   | eje Z → (x, y)
    let cx = sample_image(tex, p.z, p.y);
    let cy = sample_image(tex, p.x, p.z);
    let cz = sample_image(tex, p.x, p.y);

    cx * wx + cy * wy + cz * wz
}

// ================= Tipos de material del proyecto =================

#[derive(Clone)]
pub enum Texture {
    Image(ImageTex),
}

#[derive(Clone)]
pub struct Material {
    pub kind: MaterialKind,
}

impl Material {
    pub fn new(kind: MaterialKind) -> Self { Self { kind } }
}

#[derive(Clone)]
pub enum MaterialKind {
    /// Difuso: albedo, roughness, textura opcional
    Lambert {
        albedo: Vec3,
        roughness: f32,
        tex: Option<Texture>,
    },
    /// PBR simplificado: albedo, metallic, roughness, textura opcional
    CookTorrance {
        albedo: Vec3,
        metallic: f32,
        roughness: f32,
        tex: Option<Texture>,
    },
    /// Vidrio/plástico: albedo, IOR, transparencia, reflectividad, roughness especular
    Dielectric {
        albedo: Vec3,
        ior: f32,
        transparency: f32,
        reflectivity: f32,
        roughness: f32,
    },
}
