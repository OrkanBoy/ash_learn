use super::math::*;

pub struct Camera {
    pub position: Vector3,
    pub front_z: f32,
    pub front_x: f32,
    pub translation_speed: f32,
    pub z_x_rotation: f32,
    pub zx_y_rotation: f32,
    pub rotation_speed: f32,
    pub near_z: f32,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
// is not aligned, may cause future issues
pub struct CameraRender {
    view: Affine3,
    near_z: f32,
}

impl Camera {
    pub fn new(position: Vector3, width: f32, height: f32, near_z: f32, translation_speed: f32, rotation_speed: f32) -> Self {
        Self {
            position,
            front_z: 1.0,
            front_x: 0.0,
            translation_speed,
            z_x_rotation: 0.0,
            zx_y_rotation: 0.0,
            rotation_speed,
            near_z,
            width,
            height,
        }
    }

    pub fn update(&mut self) {
        self.front_z = self.z_x_rotation.cos();
        self.front_x = self.z_x_rotation.sin();
    }

    pub fn to_render(&self) -> CameraRender {
        let front_y_plane = Vector3::new(self.front_x, 0.0, self.front_z)
            .wedge(&Vector3::new(0.0, 1.0, 0.0));

        CameraRender {
            // scale -> rotate z to x -> rotate y to xz -> translate
            view: Affine3::IDENTITY
                .translate(&(-self.position))
                .rotate(-self.zx_y_rotation, &front_y_plane)
                .rotate(-self.z_x_rotation, &BiVector3::new(0.0, 0.0, 1.0))
                .scale(&Scale3::new(2.0 / self.width, 2.0 / self.height, 1.0)),
            near_z: self.near_z,
        }       
    }
}