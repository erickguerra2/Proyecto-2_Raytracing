use crate::math::{Vec3, Ray};
use crate::scene::{Scene, Light};
use crate::material::{Material, MaterialKind, Texture, ImageTex, CubeTex, sample_triplanar};
use crate::shapes::{Aabb, Plane};

// ============================ util ============================
fn clamp01(x: f32) -> f32 { if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x } }

// Refracción (Snell). Devuelve None si hay TIR.
fn refract(d: Vec3, n: Vec3, eta: f32) -> Option<Vec3> {
    let cosi = (-d).dot(n).max(-1.0).min(1.0);
    let (n1, n2, nn, cosi_fix) = if cosi < 0.0 {
        // estamos saliendo: invierte normal
        (eta, 1.0, -n, -cosi)
    } else {
        (1.0, eta, n,  cosi)
    };
    let eta_rel = n1 / n2;
    let k = 1.0 - eta_rel * eta_rel * (1.0 - cosi_fix * cosi_fix);
    if k < 0.0 { return None; }
    let t = d * eta_rel + nn * (eta_rel * cosi_fix - k.sqrt());
    Some(t.normalized())
}

// Muestra un pixel de ImageTex con u,v en [0,1] (clamp), sin tiling.
fn sample_imagetex_uv(tex: &ImageTex, u: f32, v: f32) -> Vec3 {
    let uu = clamp01(u);
    let vv = clamp01(v);
    let x = (uu * (tex.w as f32 - 1.0)).round() as i32;
    let y = (vv * (tex.h as f32 - 1.0)).round() as i32;
    let xi = x.clamp(0, tex.w - 1) as usize;
    let yi = y.clamp(0, tex.h - 1) as usize;
    let c = tex.pixels[yi * tex.w as usize + xi];
    Vec3::new(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
}

// Direc→cara/UV para cubemap (OpenGL-like). Devuelve (face_idx, u, v) con u,v en [0,1]
fn cubemap_face_uv(dir: Vec3) -> (usize, f32, f32) {
    let x = dir.x; let y = dir.y; let z = dir.z;
    let ax = x.abs(); let ay = y.abs(); let az = z.abs();

    // cara y coordenadas
    if ax >= ay && ax >= az {
        // ±X
        if x > 0.0 {
            // +X:  u = -z/|x|, v = -y/|x|
            let u = -z / ax;
            let v = -y / ax;
            (0, 0.5*(u+1.0), 0.5*(v+1.0))
        } else {
            // -X:  u =  z/|x|, v = -y/|x|
            let u =  z / ax;
            let v = -y / ax;
            (1, 0.5*(u+1.0), 0.5*(v+1.0))
        }
    } else if ay >= ax && ay >= az {
        // ±Y
        if y > 0.0 {
            // +Y:  u =  x/|y|, v =  z/|y|
            let u =  x / ay;
            let v =  z / ay;
            (2, 0.5*(u+1.0), 0.5*(v+1.0))
        } else {
            // -Y:  u =  x/|y|, v = -z/|y|
            let u =  x / ay;
            let v = -z / ay;
            (3, 0.5*(u+1.0), 0.5*(v+1.0))
        }
    } else {
        // ±Z
        if z > 0.0 {
            // +Z:  u =  x/|z|, v = -y/|z|
            let u =  x / az;
            let v = -y / az;
            (4, 0.5*(u+1.0), 0.5*(v+1.0))
        } else {
            // -Z:  u = -x/|z|, v = -y/|z|
            let u = -x / az;
            let v = -y / az;
            (5, 0.5*(u+1.0), 0.5*(v+1.0))
        }
    }
}

fn sample_cubemap(cube: &CubeTex, dir: Vec3) -> Vec3 {
    let (face, u, v) = cubemap_face_uv(dir);
    let tex = &cube.faces[face];
    sample_imagetex_uv(tex, u, v)
}

// ======================= Intersecciones locales =======================

#[derive(Clone)]
struct Hit {
    t: f32,
    p: Vec3,
    n: Vec3,
    mat: Material,
}

fn hit_plane(pl: &Plane, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
    let denom = pl.normal.dot(ray.d);
    if denom.abs() < 1e-6 { return None; }
    let t = (pl.point - ray.o).dot(pl.normal) / denom;
    if t < t_min || t > t_max { return None; }
    let p = ray.o + ray.d * t;
    let n = if denom < 0.0 { pl.normal } else { pl.normal * -1.0 };
    Some(Hit { t, p, n, mat: pl.mat.clone() })
}

fn hit_aabb(bx: &Aabb, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
    let invx = 1.0 / ray.d.x;
    let invy = 1.0 / ray.d.y;
    let invz = 1.0 / ray.d.z;

    let mut t0 = (bx.min.x - ray.o.x) * invx;
    let mut t1 = (bx.max.x - ray.o.x) * invx;
    let mut nx = if t0 < t1 { -1.0 } else { 1.0 };
    if t0 > t1 { std::mem::swap(&mut t0, &mut t1); }

    let mut ty0 = (bx.min.y - ray.o.y) * invy;
    let mut ty1 = (bx.max.y - ray.o.y) * invy;
    let mut ny = if ty0 < ty1 { -1.0 } else { 1.0 };
    if ty0 > ty1 { std::mem::swap(&mut ty0, &mut ty1); }

    if t0 > ty1 || ty0 > t1 { return None; }
    let mut t_near = if t0 > ty0 { t0 } else { ty0 };
    let mut t_far  = if t1 < ty1 { t1 } else { ty1 };
    let mut n = if t0 > ty0 { Vec3::new(nx,0.0,0.0) } else { Vec3::new(0.0,ny,0.0) };

    let mut tz0 = (bx.min.z - ray.o.z) * invz;
    let mut tz1 = (bx.max.z - ray.o.z) * invz;
    let mut nz = if tz0 < tz1 { -1.0 } else { 1.0 };
    if tz0 > tz1 { std::mem::swap(&mut tz0, &mut tz1); }

    if t_near > tz1 || tz0 > t_far { return None; }
    if tz0 > t_near { t_near = tz0; n = Vec3::new(0.0,0.0,nz); }
    if tz1 < t_far  { t_far  = tz1; }

    if t_near < t_min || t_near > t_max { return None; }
    let t = t_near.max(t_min);
    let p = ray.o + ray.d * t;
    Some(Hit { t, p, n, mat: bx.mat.clone() })
}

fn hit_scene(scene: &Scene, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    let mut closest = t_max;

    for pl in &scene.planes {
        if let Some(h) = hit_plane(pl, ray, t_min, closest) {
            closest = h.t;
            best = Some(h);
        }
    }
    for bx in &scene.boxes_ {
        if let Some(h) = hit_aabb(bx, ray, t_min, closest) {
            closest = h.t;
            best = Some(h);
        }
    }
    best
}

// ======================= Shading =======================

fn base_color_with_tex(kind: &MaterialKind, p: &Vec3, n: &Vec3) -> Vec3 {
    let (albedo, tex_opt) = match kind {
        MaterialKind::Lambert { albedo, tex, .. } => (*albedo, tex),
        MaterialKind::CookTorrance { albedo, tex, .. } => (*albedo, tex),
        MaterialKind::Dielectric { albedo, .. } => (*albedo, &None),
    };
    let mut b = albedo;
    if let Some(Texture::Image(img)) = tex_opt {
        // Triplanar en mundo
        b = b * sample_triplanar(img, p, n);
    }
    b
}

fn shade_lights(scene: &Scene, n: Vec3, p: Vec3, v: Vec3, base: Vec3, kind: &MaterialKind) -> Vec3 {
    let mut c = Vec3::new(0.0, 0.0, 0.0);

    match kind {
        MaterialKind::Lambert { .. } => {
            for l in &scene.lights {
                match l {
                    Light::Ambient(a) => { c = c + base * *a; }
                    Light::Directional { dir, intensity } => {
                        let ldir = (-*dir).normalized();
                        let ndl = clamp01(n.dot(ldir));
                        c = c + base * (*intensity * ndl);
                    }
                }
            }
        }
        MaterialKind::CookTorrance { metallic, roughness, .. } => {
            for l in &scene.lights {
                match l {
                    Light::Ambient(a) => { c = c + base * *a; }
                    Light::Directional { dir, intensity } => {
                        let ldir = (-*dir).normalized();
                        let ndl = clamp01(n.dot(ldir));
                        let h = (ldir + v).normalized();
                        let ndh = clamp01(n.dot(h));
                        // especular muy simple:
                        let spec_str = (1.0 - *roughness).powf(2.0);
                        let spec = spec_str * ndh.powf(32.0);
                        let f0 = 0.04 * (1.0 - *metallic) + *metallic;
                        let spec_col = Vec3::one() * (spec * f0);
                        c = c + (base * (*intensity * ndl) + spec_col * *intensity);
                    }
                }
            }
        }
        MaterialKind::Dielectric { .. } => {
            // iluminación base tenue; el resto lo maneja mezcla reflect/refract
            for l in &scene.lights {
                if let Light::Ambient(a) = l {
                    c = c + base * (*a * 0.25);
                }
            }
        }
    }

    c
}

pub fn ray_color(scene: &Scene, ray: &Ray, depth: i32) -> Vec3 {
    if depth > 3 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    if let Some(h) = hit_scene(scene, ray, 1e-3, 1e9) {
        let v = (-ray.d).normalized();
        let base = base_color_with_tex(&h.mat.kind, &h.p, &h.n);

        match &h.mat.kind {
            MaterialKind::Dielectric { albedo, ior, transparency, reflectivity, roughness: _ } => {
                // reflect ray
                let rdir = ray.d.reflect(h.n).normalized();
                let cr = ray_color(scene, &Ray { o: h.p + rdir * 1e-3, d: rdir }, depth + 1);

                // refract ray
                let ct = if let Some(tdir) = refract(ray.d, h.n, *ior) {
                    ray_color(scene, &Ray { o: h.p + tdir * 1e-3, d: tdir }, depth + 1)
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                };

                let local = shade_lights(scene, h.n, h.p, v, base * *albedo, &h.mat.kind);
                local * 0.1 + cr * *reflectivity + ct * *transparency
            }
            MaterialKind::CookTorrance { .. } => {
                shade_lights(scene, h.n, h.p, v, base, &h.mat.kind)
            }
            MaterialKind::Lambert { .. } => {
                shade_lights(scene, h.n, h.p, v, base, &h.mat.kind)
            }
        }
    } else {
        // Fondo: skybox si existe, si no, cielo celeste
        if let Some(cube) = &scene.skybox {
            sample_cubemap(cube, ray.d)
        } else {
            // degradé simple
            let t = 0.5 * (ray.d.y + 1.0);
            Vec3::new(0.5, 0.7, 1.0) * t + Vec3::new(0.8, 0.9, 1.0) * (1.0 - t)
        }
    }
}
