use raylib::prelude::*;

pub struct Camera {
    pub eye: Vector3,
    pub center: Vector3,
    pub up: Vector3,
    pub forward: Vector3,
    pub right: Vector3,
}

impl Camera {
    pub fn new(eye: Vector3, center: Vector3, up: Vector3) -> Self {
        let mut c = Self {
            eye, center, up,
            forward: Vector3::zero(),
            right: Vector3::zero(),
        };
        c.update_basis_vectors();
        c
    }

    pub fn update_basis_vectors(&mut self) {
        self.forward = (self.center - self.eye).normalized();
        self.right = self.forward.cross(self.up).normalized();
        self.up = self.right.cross(self.forward);
    }

    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let rel = self.eye - self.center;
        let r = rel.length();
        let cur_yaw = rel.z.atan2(rel.x);
        let cur_pitch = (rel.y / r).asin();

        let ny = cur_yaw + yaw;
        let np = (cur_pitch + pitch).clamp(-1.45, 1.45);

        let cp = np.cos();
        let new_rel = Vector3::new(r * cp * ny.cos(), r * np.sin(), r * cp * ny.sin());
        self.eye = self.center + new_rel;
        self.update_basis_vectors();
    }

    pub fn dolly(&mut self, amount: f32) {
        let dir = self.forward;
        let new_eye = self.eye + dir * amount;
        // evita atravesar el centro
        if (new_eye - self.center).length() > 0.2 {
            self.eye = new_eye;
            self.update_basis_vectors();
        }
    }

    /// pasa de coords cámara a mundo (base derecha, arriba, -forward)
    pub fn basis_change(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            v.x * self.right.x + v.y * self.up.x - v.z * self.forward.x,
            v.x * self.right.y + v.y * self.up.y - v.z * self.forward.y,
            v.x * self.right.z + v.y * self.up.z - v.z * self.forward.z,
        )
    }
}
