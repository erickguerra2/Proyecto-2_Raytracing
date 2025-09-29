use raylib::prelude::*;
use std::f32::consts::PI;

mod camera;
mod framebuffer;
mod material;
mod ray_intersect;
mod cube;
mod light;
mod texture;
mod skybox;

use camera::Camera;
use framebuffer::Framebuffer;
use material::{Material, v3_to_color};
use ray_intersect::{Intersect, RayIntersect, reflect, refract, offset_origin};
use cube::Cube;
use light::Light;
use texture::TextureCPU;
use skybox::Skybox;

// === sombreado ===
fn phong_shade(hit: &Intersect, light: &Light, view_dir: Vector3) -> (Vector3 /*kd*/, f32 /*spec*/) {
    let ldir = (light.pos - hit.point).normalized();
    let ndotl = hit.normal.dot(ldir).max(0.0);

    // componemos kd por componente (evita Vector3 * Vector3 directo)
    let base = hit.mat.diffuse * (ndotl * light.intensity); // Vector3 * escalar
    let lc = light.color; // Vector3 (1,1,1) o el color de la luz
    let kd = Vector3::new(base.x * lc.x, base.y * lc.y, base.z * lc.z);

    let r = reflect(&-ldir, &hit.normal).normalized();
    let spec = view_dir.dot(r).max(0.0).powf(hit.mat.specular_exp);
    (kd, spec)
}

fn cast_ray(
    ro: &Vector3, rd: &Vector3,
    objects: &[Box<dyn RayIntersect + Sync>],
    light: &Light,
    sky: &Skybox,
    tex_albedo: &dyn Fn(&Intersect)->Vector3,
    depth: u32
) -> Vector3 {
    if depth > 3 { return sky.sample(*rd); }

    let mut best = Intersect::empty();
    let mut z = f32::INFINITY;
    for o in objects {
        let i = o.ray_intersect(ro, rd);
        if i.hit && i.t < z { z = i.t; best = i; }
    }
    if !best.hit { return sky.sample(*rd); }

    // texturas (albedo multiplicativo)
    let base_tex = tex_albedo(&best);
    let view_dir = (*ro - best.point).normalized();
    let (kd_col, spec_sc) = phong_shade(&best, light, view_dir);
    let kd = Vector3::new(
        kd_col.x * base_tex.x,
        kd_col.y * base_tex.y,
        kd_col.z * base_tex.z
    );
    let ks = Vector3::new(spec_sc, spec_sc, spec_sc) * light.intensity;

    // componentes
    let (ka, ks_w, kr, kt) = (best.mat.albedo[0], best.mat.albedo[1], best.mat.albedo[2], best.mat.albedo[3]);

    let mut color = kd * ka + ks * ks_w;

    // reflexión
    if kr > 0.0 {
        let rdir = reflect(rd, &best.normal).normalized();
        let rorig = offset_origin(&best.point, &best.normal, &rdir);
        let rc = cast_ray(&rorig, &rdir, objects, light, sky, tex_albedo, depth+1);
        color = color*(1.0-kr) + rc*kr;
    }

    // refracción
    if kt > 0.0 {
        if let Some(tdir) = refract(rd, &best.normal, best.mat.ior) {
            let torig = offset_origin(&best.point, &best.normal, &tdir);
            let tc = cast_ray(&torig, &tdir, objects, light, sky, tex_albedo, depth+1);
            color = color*(1.0-kt) + tc*kt;
        } else {
            // TIR: ya lo maneja la reflexión de arriba
        }
    }

    color
}

fn render(
    fb: &mut Framebuffer,
    cam: &Camera,
    light: &Light,
    sky: &Skybox,
    objects: &[Box<dyn RayIntersect + Sync>],
    tex_albedo: &dyn Fn(&Intersect)->Vector3
) {
    let w = fb.width as f32;
    let h = fb.height as f32;
    let aspect = w/h;
    let fov = PI/3.0;
    let scale = (fov*0.5).tan();

    for y in 0..fb.height {
        for x in 0..fb.width {
            let sx = (2.0 * x as f32) / w - 1.0;
            let sy = -(2.0 * y as f32) / h + 1.0;
            let sx = sx * aspect * scale;
            let sy = sy * scale;

            let rd_cam = Vector3::new(sx, sy, -1.0).normalized();
            let rd = cam.basis_change(&rd_cam).normalized();
            let col = cast_ray(&cam.eye, &rd, objects, light, sky, tex_albedo, 0);

            fb.set_current_color(v3_to_color(col));
            fb.set_pixel(x, y);
        }
    }
}

// === escena: casa sencilla con 5 materiales + agua refractiva y vidrio reflectivo ===
fn main() {
    let (mut rl, th) = raylib::init()
        .size(960, 540)
        .title("Diorama Raytracer — Casa sencilla")
        .build();

    // carga texturas CPU
    let img_brick = Image::load_image("assets/textures/brick.png").expect("brick.png");
    let img_wood  = Image::load_image("assets/textures/wood.png").expect("wood.png");
    let img_quartz= Image::load_image("assets/textures/quartz.png").expect("quartz.png");
    let img_glass = Image::load_image("assets/textures/glass.png").expect("glass.png");
    let img_water = Image::load_image("assets/textures/water.png").expect("water.png");

    let tex_brick = TextureCPU::from_image(&img_brick).unwrap();
    let tex_wood  = TextureCPU::from_image(&img_wood).unwrap();
    let tex_quartz= TextureCPU::from_image(&img_quartz).unwrap();
    let tex_glass = TextureCPU::from_image(&img_glass).unwrap();
    let tex_water = TextureCPU::from_image(&img_water).unwrap();

    // skybox
    let sky = Skybox::new(
        TextureCPU::from_image(&Image::load_image("assets/sky/nx.png").unwrap()).unwrap(),
        TextureCPU::from_image(&Image::load_image("assets/sky/px.png").unwrap()).unwrap(),
        TextureCPU::from_image(&Image::load_image("assets/sky/ny.png").unwrap()).unwrap(),
        TextureCPU::from_image(&Image::load_image("assets/sky/py.png").unwrap()).unwrap(),
        TextureCPU::from_image(&Image::load_image("assets/sky/nz.png").unwrap()).unwrap(),
        TextureCPU::from_image(&Image::load_image("assets/sky/pz.png").unwrap()).unwrap(),
    );

    // materiales (kd, shininess, [kd,ks,kr,kt], ior)
    let mat_brick  = Material::new(Vector3::new(0.9,0.9,0.9), 32.0, [0.9,0.1,0.0,0.0], 1.0);
    let mat_wood   = Material::new(Vector3::new(0.9,0.8,0.7), 32.0, [0.95,0.05,0.0,0.0], 1.0);
    let mat_quartz = Material::new(Vector3::new(1.0,1.0,1.0), 64.0, [0.8,0.2,0.0,0.0], 1.0);
    let mat_glass  = Material::new(Vector3::new(1.0,1.0,1.0), 96.0, [0.1,0.3,0.4,0.4], 1.5); // reflexión + refracción
    let mat_water  = Material::new(Vector3::new(0.8,0.9,1.0), 16.0, [0.2,0.1,0.05,0.65], 1.33);

    // construye casa (tamaño controlado)
    let mut objects: Vec<Box<dyn RayIntersect + Sync>> = Vec::new();

    // plataforma (cuarzo) – más “baldozas”
    objects.push(Box::new(
        Cube::from_center_size(
            Vector3::new(0.0,-0.55, 0.0), 
            Vector3::new(6.0,0.5,6.0), 
            mat_quartz
        ).with_tiling(5.0) // <-- repite 5x
    ));

    // paredes (ladrillo)
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(0.0, 0.5, -1.5), Vector3::new(3.0, 2.0, 0.2), mat_brick)
            .with_tiling(3.5)
    ));
    // frontal izquierda/derecha
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(-0.9, 0.5, 1.5), Vector3::new(1.2, 2.0, 0.2), mat_brick)
            .with_tiling(3.5)
    ));
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new( 0.9, 0.5, 1.5), Vector3::new(1.2, 2.0, 0.2), mat_brick)
            .with_tiling(3.5)
    ));
    // laterales
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(-1.5, 0.5, 0.0), Vector3::new(0.2, 2.0, 3.2), mat_brick)
            .with_tiling(3.5)
    ));
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new( 1.5, 0.5, 0.0), Vector3::new(0.2, 2.0, 3.2), mat_brick)
            .with_tiling(3.5)
    ));

    // techo (madera) – mucho tiling para vetas finas
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(0.0, 1.6, 0.0), Vector3::new(3.4, 0.2, 3.6), mat_wood)
            .with_tiling(6.0)
    ));

    // ventanas (cristal) – 1:1 o un poco de tiling si tu textura lo permite
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(0.0, 0.8, -1.4), Vector3::new(1.2, 0.8, 0.05), mat_glass)
            .with_tiling(1.5)
    ));
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(-1.4, 0.8, 0.0), Vector3::new(0.05, 0.8, 1.0), mat_glass)
            .with_tiling(1.5)
    ));

    // agua – un tiling moderado
    objects.push(Box::new(
        Cube::from_center_size(Vector3::new(0.0, -0.49, 2.6), Vector3::new(1.8, 0.12, 1.2), mat_water)
            .with_tiling(2.5)
    ));


    // función para muestrear albedo texturizado por material
    let albedo_fn = move |hit: &Intersect| -> Vector3 {
        let (u,v) = hit.uv;
        // el tinte de material multiplica la textura
        let tint = hit.mat.diffuse;
        // decide cuál textura usar (sencillo: por puntero de ior/albedo)
        if (hit.mat.ior - 1.5).abs() < 0.01 { return tex_glass.sample_repeat(u,v) * tint; }
        if (hit.mat.ior - 1.33).abs() < 0.02 { return tex_water.sample_repeat(u,v) * tint; }
        // compara por ks alto? aquí por afinidad:
        if hit.mat.specular_exp >= 60.0 && hit.mat.albedo[1] >= 0.2 { return tex_quartz.sample_repeat(u,v) * tint; }
        // ladrillo vs madera: heurística por tamaño del bloque en Y (techo delgado → madera)
        if hit.normal.y.abs() > 0.9 && hit.mat.albedo[0] > 0.9 && hit.mat.specular_exp < 40.0 {
            return tex_wood.sample_repeat(u,v) * tint;
        }
        tex_brick.sample_repeat(u,v) * tint
    };

    // luz
    let light = Light::new(
        Vector3::new(2.5, 3.0, 3.0),
        Vector3::new(1.0, 1.0, 1.0),
        1.5
    );

    // cámara
    let mut cam = Camera::new(
        Vector3::new(4.0, 2.2, 5.0),
        Vector3::new(0.0, 0.6, 0.0),
        Vector3::new(0.0, 1.0, 0.0)
    );

    let mut fb = Framebuffer::new(960, 540);

    rl.set_target_fps(30);
    while !rl.window_should_close() {
        // Controles:
        // ← → → orbita yaw
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { cam.orbit( 0.02, 0.0); }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam.orbit(-0.02, 0.0); }
        // ↑ ↓ → orbita pitch
        if rl.is_key_down(KeyboardKey::KEY_UP)    { cam.orbit(0.0, -0.02); }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { cam.orbit(0.0,  0.02); }
        // Zoom dolly (W/S)
        if rl.is_key_down(KeyboardKey::KEY_W)     { cam.dolly( 0.10); }
        if rl.is_key_down(KeyboardKey::KEY_S)     { cam.dolly(-0.10); }
        // Guardar frame (P)
        if rl.is_key_pressed(KeyboardKey::KEY_P)  { fb.save_png("frame.png"); }

        fb.clear();
        render(&mut fb, &cam, &light, &sky, &objects, &albedo_fn);
        fb.blit(&mut rl, &th);
    }
}
