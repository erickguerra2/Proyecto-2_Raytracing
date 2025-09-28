use std::os::raw::c_void;
use std::time::Instant;

use raylib::prelude::*;
use raylib::ffi;
use rayon::prelude::*;

use crate::math::{Vec3, Ray};
use crate::material::{Material, MaterialKind, Texture, load_image_tex, load_cubemap};
use crate::scene::{Scene, Light};
use crate::shapes::{Aabb, Plane};

mod math;
mod material;
mod scene;
mod shapes;
mod shading;

// =================== Config ventana / render ===================
const WINDOW_W: i32 = 1280;
const WINDOW_H: i32 = 720;

// Render interno (más chico = más FPS). Se ajusta dinámicamente.
const RENDER_W: usize = 960;
const RENDER_H: usize = 540;

// Cámara (control)
const CAM_FOV_DEG: f32   = 55.0;
const CAM_AZIM_SPEED: f32 = 2.2;  // rad/s   (izq-der)
const CAM_PAN_Y_SPEED: f32 = 3.2; // u/s     (arr-aba)
const CAM_ZOOM_SPEED: f32  = 4.0; // u/s     (PgUp/PgDn)

// Calidad dinámica (F1/F2). Min bajado a 0.45 para más FPS si hace falta.
static mut QUALITY_SCALE: f32 = 1.0; // 0.45..1.0

// =================== Cámara orbital con pan vertical ===================
struct OrbitalCam {
    target_base: Vec3, // punto base (centro del SLS)
    y_ofs: f32,        // pan vertical absoluto (mueve cámara y target)
    distance: f32,     // radio de la órbita (zoom)
    azim: f32,         // ángulo horizontal (rotación alrededor del SLS)
    elev: f32,         // inclinación fija pequeña para ver desde arriba
    fov_y: f32,        // grados
    aspect: f32,
}
impl OrbitalCam {
    fn new(target: Vec3, distance: f32, azim: f32, elev: f32, fov_y: f32, aspect: f32) -> Self {
        Self { target_base: target, y_ofs: 0.0, distance, azim, elev, fov_y, aspect }
    }
    fn target(&self) -> Vec3 {
        self.target_base + Vec3::new(0.0, self.y_ofs, 0.0)
    }
    fn eye(&self) -> Vec3 {
        let tgt = self.target();
        let ce = self.elev.cos();
        let x = self.distance * ce * self.azim.cos();
        let z = self.distance * ce * self.azim.sin();
        let y = self.distance * self.elev.sin();
        tgt + Vec3::new(x, y, z)
    }
    /// Devuelve (origen, base_u, base_v, forward) para construir rays
    fn frame(&self) -> (Vec3, Vec3, Vec3, Vec3) {
        let eye = self.eye();
        let tgt = self.target();
        let f = (tgt - eye).normalized();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let r = f.cross(up).normalized();
        let u2 = r.cross(f).normalized();

        let tan_fov = (self.fov_y.to_radians() * 0.5).tan();
        let half_v = tan_fov;
        let half_h = tan_fov * self.aspect;

        let base_u = r * half_h;
        let base_v = u2 * half_v;

        (eye, base_u, base_v, f)
    }
}

// =================== App ===================
fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_W, WINDOW_H)
        .title("SLS Diorama (Raytracing + Rayon) - UVG")
        .build();
    rl.set_target_fps(60);

    // Imagen negra inicial
    let img0 = Image::gen_image_color(RENDER_W as i32, RENDER_H as i32, Color::BLACK);
    let tex = rl.load_texture_from_image(&thread, &img0).expect("texture");

    // ---------- Escena ----------
    let mut scene = Scene::new();

    // Skybox (usa tus 6 archivos: px/nx/py/ny/pz/nz)
    scene.skybox = Some(load_cubemap(
        "textures/skybox/px.png",
        "textures/skybox/nx.png",
        "textures/skybox/py.png",
        "textures/skybox/ny.png",
        "textures/skybox/pz.png",
        "textures/skybox/nz.png",
    ));

    // Luces
    scene.lights.push(Light::Ambient(Vec3::new(0.25, 0.25, 0.25)));
    scene.lights.push(Light::Directional {
        dir: Vec3::new(-0.7, -1.0, -0.35).normalized(),
        intensity: Vec3::new(1.25, 1.25, 1.25),
    });

    // ======== Materiales (texturas activas donde conviene) ========
    let mat_grass = Material::new(MaterialKind::Lambert {
        albedo: Vec3::new(0.18, 0.49, 0.20),
        roughness: 0.97,
        tex: None,
    });
    let mat_concrete = Material::new(MaterialKind::Lambert {
        albedo: Vec3::new(0.80, 0.80, 0.80),
        roughness: 0.95,
        tex: Some(Texture::Image(load_image_tex("textures/pad_concrete.png", 1.6))),
    });
    let mat_tower = Material::new(MaterialKind::CookTorrance {
        albedo: Vec3::new(0.85, 0.12, 0.12),
        metallic: 0.85,
        roughness: 0.45,
        tex: Some(Texture::Image(load_image_tex("textures/tower_red.png", 2.0))),
    });
    let mat_orange = Material::new(MaterialKind::CookTorrance {
        albedo: Vec3::new(0.90, 0.58, 0.18),
        metallic: 0.05,
        roughness: 0.62,
        tex: Some(Texture::Image(load_image_tex("textures/rocket_orange.png", 2.2))),
    });
    let mat_white = Material::new(MaterialKind::CookTorrance {
        albedo: Vec3::new(0.95, 0.95, 0.98),
        metallic: 0.04,
        roughness: 0.42,
        tex: Some(Texture::Image(load_image_tex("textures/rocket_white.png", 2.0))),
    });
    let mat_black = Material::new(MaterialKind::CookTorrance {
        albedo: Vec3::new(0.08, 0.07, 0.07),
        metallic: 0.75,
        roughness: 0.22,
        tex: Some(Texture::Image(load_image_tex("textures/rocket_black.png", 2.0))),
    });
    let mat_window = Material::new(MaterialKind::Dielectric {
        albedo: Vec3::new(0.94, 0.97, 1.0),
        ior: 1.50,
        transparency: 0.82,
        reflectivity: 0.10,
        roughness: 0.0,
    });
    let mat_decal = Material::new(MaterialKind::Lambert {
        albedo: Vec3::one(),
        roughness: 0.9,
        tex: Some(Texture::Image(load_image_tex("textures/rocket_decal.png", 1.0))),
    });

    // ---------- Geometría SLS ----------
    build_sls_scene(
        &mut scene,
        &mat_grass,
        &mat_concrete,
        &mat_tower,
        &mat_orange,
        &mat_white,
        &mat_black,
        &mat_window,
        &mat_decal,
    );

    // ---------- Cámara: más lejos + pan Y por flechas ↑/↓ ----------
    let target = Vec3::new(0.0, 3.2, 0.0);
    let mut cam = OrbitalCam::new(
        target,
        /* distance */ 15.5,  // ← más alejada
        /* azim  */ 1.05,
        /* elev  */ 0.35,     // pitch fijo suave
        /* fov_y */ CAM_FOV_DEG,
        /* aspect */ RENDER_W as f32 / RENDER_H as f32,
    );

    // Buffer final (siempre tamaño RENDER_W/H)
    let mut rgba: Vec<u8> = vec![0; RENDER_W * RENDER_H * 4];

    // Auto–resolución (objetivo ~30 FPS más agresivo)
    let mut moving_avg_ms = 30.0_f32;

    while !rl.window_should_close() {
        // -------- Input suave --------
        let dt = rl.get_frame_time();
        let boost = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) { 2.2 } else { 1.0 };

        // ORBITA (izq/der)
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam.azim += CAM_AZIM_SPEED * boost * dt; }
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { cam.azim -= CAM_AZIM_SPEED * boost * dt; }

        // PAN VERTICAL (arr/aba)
        if rl.is_key_down(KeyboardKey::KEY_UP)    { cam.y_ofs += CAM_PAN_Y_SPEED * boost * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { cam.y_ofs -= CAM_PAN_Y_SPEED * boost * dt; }
        // Limita el pan vertical para no perder el SLS de vista
        if cam.y_ofs < -2.0 { cam.y_ofs = -2.0; }
        if cam.y_ofs >  6.0 { cam.y_ofs =  6.0; }

        // ZOOM (PgUp/PgDn y +/- numpad)
        if rl.is_key_down(KeyboardKey::KEY_PAGE_UP) || rl.is_key_down(KeyboardKey::KEY_EQUAL) || rl.is_key_down(KeyboardKey::KEY_KP_ADD) {
            cam.distance -= CAM_ZOOM_SPEED * boost * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_PAGE_DOWN) || rl.is_key_down(KeyboardKey::KEY_MINUS) || rl.is_key_down(KeyboardKey::KEY_KP_SUBTRACT) {
            cam.distance += CAM_ZOOM_SPEED * boost * dt;
        }
        if cam.distance < 7.0  { cam.distance = 7.0; }
        if cam.distance > 24.0 { cam.distance = 24.0; }

        unsafe {
            if rl.is_key_pressed(KeyboardKey::KEY_F1) { QUALITY_SCALE = 0.60; } // rápido
            if rl.is_key_pressed(KeyboardKey::KEY_F2) { QUALITY_SCALE = 1.00; } // calidad
        }

        // -------- Ray frame de la cámara --------
        let (cam_o, mut base_u, base_v, fwd) = cam.frame();

        // FIX del flip horizontal:
        base_u = base_u * -1.0;

        // -------- Render paralelo + downscale --------
        let (eff_w, eff_h) = unsafe {
            let s = QUALITY_SCALE.clamp(0.45, 1.0);
            (((RENDER_W as f32)*s) as usize, ((RENDER_H as f32)*s) as usize)
        };

        let mut small = vec![0u8; eff_w * eff_h * 4];

        let t0 = Instant::now();

        small.par_chunks_mut(4).enumerate().for_each(|(i, px)| {
            let x = (i % eff_w) as i32;
            let y = (i / eff_w) as i32;

            // Coordenadas normalizadas
            let u = (x as f32 + 0.5) / eff_w as f32;
            let v = (y as f32 + 0.5) / eff_h as f32;

            // FIX del flip vertical (raylib dibuja con origen arriba-izquierda):
            let v_img = 1.0 - v;

            let dir = (fwd + base_u * (u * 2.0 - 1.0) + base_v * (v_img * 2.0 - 1.0)).normalized();
            let ray = Ray { o: cam_o, d: dir };

            let c = shading::ray_color(&scene, &ray, 0);

            px[0] = (c.x.clamp(0.0, 1.0) * 255.0) as u8;
            px[1] = (c.y.clamp(0.0, 1.0) * 255.0) as u8;
            px[2] = (c.z.clamp(0.0, 1.0) * 255.0) as u8;
            px[3] = 255;
        });

        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        // EWMA simple
        moving_avg_ms = moving_avg_ms * 0.85 + ms * 0.15;

        // Auto–resolución para mantener ~30 ms (≈33 FPS) algo más agresivo
        unsafe {
            let target_ms = 30.0_f32;
            let mut s = QUALITY_SCALE;
            if moving_avg_ms > target_ms * 1.08 && s > 0.45 {
                s *= 0.92; // baja un poco
            } else if moving_avg_ms < target_ms * 0.88 && s < 1.0 {
                s *= 1.04; // sube un poquito
            }
            QUALITY_SCALE = s.clamp(0.45, 1.0);
        }

        // Upscale si fue render parcial
        let mut rgba = &mut rgba; // alias local para borrow
        if eff_w != RENDER_W || eff_h != RENDER_H {
            for y in 0..RENDER_H {
                let ys = (y * eff_h) / RENDER_H;
                for x in 0..RENDER_W {
                    let xs = (x * eff_w) / RENDER_W;
                    let src = (ys * eff_w + xs) * 4;
                    let dst = (y * RENDER_W + x) * 4;
                    rgba[dst..dst + 4].copy_from_slice(&small[src..src + 4]);
                }
            }
        } else {
            rgba.copy_from_slice(&small);
        }

        // Actualizar textura sin mover `tex`
        unsafe {
            ffi::UpdateTexture(*tex.as_ref(), rgba.as_ptr() as *const c_void);
        }

        // -------- Dibujo (escalado a la ventana completa) --------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        let src = Rectangle { x: 0.0, y: 0.0, width: RENDER_W as f32, height: RENDER_H as f32 };
        let dst = Rectangle { x: 0.0, y: 0.0, width: WINDOW_W as f32, height: WINDOW_H as f32 };
        let origin = Vector2 { x: 0.0, y: 0.0 };
        d.draw_texture_pro(&tex, src, dst, origin, 0.0, Color::WHITE);

        d.draw_text("←/→: rotar | ↑/↓: pan vertical | PgUp/PgDn (+/-): zoom | Shift: rápido | F1/F2: calidad",
                    12, 12, 18, Color::RAYWHITE);
    }
}

// =================== Construcción de escena ===================

fn build_sls_scene(
    scene: &mut Scene,
    mat_grass: &Material,
    mat_concrete: &Material,
    mat_tower: &Material,
    mat_orange: &Material,
    mat_white: &Material,
    mat_black: &Material,
    mat_window: &Material,
    mat_decal: &Material,
) {
    // --- Suelo plano (rápido) ---
    scene.planes.push(Plane {
        point: Vec3::new(0.0, 0.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        mat: mat_grass.clone(),
    });

    // --- Plataforma de concreto (contenida para rendimiento) ---
    let pad_y0 = 0.0;
    let pad_y1 = 0.5;
    scene.add_box(Aabb::from_min_max(Vec3::new(-4.5, pad_y0, -4.5), Vec3::new( 4.5, pad_y1,  4.5), mat_concrete.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-3.2, pad_y1, -3.2), Vec3::new( 3.2, pad_y1+0.6, 3.2), mat_concrete.clone()));

    // --- Torre (delgada y desplazada para no tapar) ---
    let tower_x = 4.7;
    let tower_h = 9.0;
    scene.add_box(Aabb::from_min_max(Vec3::new(tower_x-0.25, pad_y1,   -0.5),
                                     Vec3::new(tower_x+0.25, pad_y1+tower_h, 0.5), mat_tower.clone()));
    // brazos simples
    scene.add_box(Aabb::from_min_max(Vec3::new(tower_x-0.25, pad_y1+5.2,  0.5),
                                     Vec3::new(2.4,          pad_y1+5.5,  0.9), mat_tower.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(tower_x-0.25, pad_y1+7.1, -0.9),
                                     Vec3::new(2.6,          pad_y1+7.4, -0.5), mat_tower.clone()));

    // --- SLS dimensiones aproximadas (escala “voxel”) ---
    let base_y  = pad_y1 + 0.6;
    let core_r  = 1.15;
    let core_h  = 8.6;

    // Core naranja
    scene.add_box(Aabb::from_min_max(Vec3::new(-core_r, base_y, -core_r),
                                     Vec3::new( core_r, base_y+core_h, core_r), mat_orange.clone()));

    // “Intertank” banda negra
    scene.add_box(Aabb::from_min_max(Vec3::new(-core_r, base_y+2.2, -core_r),
                                     Vec3::new( core_r, base_y+2.5,  core_r), mat_black.clone()));

    // ICPS / etapa superior (blanca)
    let upper_h = 1.5;
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.9, base_y+core_h, -0.9),
                                     Vec3::new( 0.9, base_y+core_h+upper_h, 0.9), mat_white.clone()));

    // Adaptador + Orion/LES (negro/blanco)
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.65, base_y+core_h+upper_h, -0.65),
                                     Vec3::new( 0.65, base_y+core_h+upper_h+0.9, 0.65), mat_black.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.40, base_y+core_h+upper_h+0.9, -0.40),
                                     Vec3::new( 0.40, base_y+core_h+upper_h+1.6, 0.40), mat_white.clone()));
    // punta LES
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.18, base_y+core_h+upper_h+1.6, -0.18),
                                     Vec3::new( 0.18, base_y+core_h+upper_h+2.2, 0.18), mat_black.clone()));

    // Ventanas (4 caras)
    let win_y0 = base_y + core_h + 0.25;
    let win_y1 = win_y0 + 0.45;
    let rc = core_r;
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.55, win_y0, rc-0.10), Vec3::new( 0.55, win_y1, rc+0.10), mat_window.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.55, win_y0,-rc-0.10), Vec3::new( 0.55, win_y1,-rc+0.10), mat_window.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new( rc-0.10, win_y0,-0.55), Vec3::new( rc+0.10, win_y1, 0.55), mat_window.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-rc-0.10, win_y0,-0.55), Vec3::new(-rc+0.10, win_y1, 0.55), mat_window.clone()));

    // Decals (4 tiras perimetrales)
    let dec_y0 = base_y + 1.3;
    let dec_y1 = dec_y0 + 0.8;
    scene.add_box(Aabb::from_min_max(Vec3::new(-rc,     dec_y0,  rc+0.01), Vec3::new( rc,     dec_y1,  rc+0.10), mat_decal.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-rc,     dec_y0, -rc-0.10), Vec3::new( rc,     dec_y1, -rc-0.01), mat_decal.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new( rc+0.01,dec_y0, -rc     ), Vec3::new( rc+0.10,dec_y1,  rc     ), mat_decal.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-rc-0.10,dec_y0, -rc     ), Vec3::new(-rc-0.01,dec_y1,  rc     ), mat_decal.clone()));

    // Motor mount (negro)
    scene.add_box(Aabb::from_min_max(Vec3::new(-0.50, base_y-0.55, -0.50),
                                     Vec3::new( 0.50, base_y,       0.50), mat_black.clone()));

    // SRBs (dos boosters, blancos)
    let srb_off = 2.4;
    let srb_r   = 0.55;
    let srb_h   = 8.2;
    let srb_blocks = [
        (-srb_r, -srb_r,  srb_off,      srb_r,  srb_r,  srb_off+0.8),
        (-srb_r, -srb_r, -srb_off-0.8,  srb_r,  srb_r, -srb_off  ),
        ( srb_off, -srb_r,-srb_r,  srb_off+0.8, srb_r,  srb_r),
        (-srb_off-0.8, -srb_r,-srb_r,  -srb_off,      srb_r,  srb_r),
    ];
    for (x0,_z0,y0,x1,_z1,y1) in srb_blocks {
        scene.add_box(Aabb::from_min_max(
            Vec3::new(x0, base_y, y0),
            Vec3::new(x1, base_y+srb_h, y1),
            mat_white.clone(),
        ));
    }
    // narices de SRB
    scene.add_box(Aabb::from_min_max(Vec3::new( srb_off+0.15, base_y+srb_h, -0.25),
                                     Vec3::new( srb_off+0.55, base_y+srb_h+0.7, 0.25), mat_black.clone()));
    scene.add_box(Aabb::from_min_max(Vec3::new(-srb_off-0.55, base_y+srb_h, -0.25),
                                     Vec3::new(-srb_off-0.15, base_y+srb_h+0.7, 0.25), mat_black.clone()));

    // costillas en el core (detalle leve, barato)
    for k in 0..6 {
        let t0 = base_y + 0.4 + k as f32 * 1.3;
        scene.add_box(Aabb::from_min_max(Vec3::new(-core_r, t0, -core_r-0.08),
                                         Vec3::new( core_r, t0+0.07, -core_r+0.08), mat_orange.clone()));
        scene.add_box(Aabb::from_min_max(Vec3::new(-core_r, t0,  core_r-0.08),
                                         Vec3::new( core_r, t0+0.07,  core_r+0.08), mat_orange.clone()));
    }
}
