use super::math::*;

pub struct Camera {
    position: Vector3,
    z_x_rotation: f32,
    y_zx_rotation: f32,
    z_near: f32,
    width: f32,
    height: f32,
}

#[repr(C)]
pub struct CameraRender {
    view: Affine3,
}

impl Camera {
    pub fn to_render(&self) -> CameraRender {
        let front_y_plane = Vector3::new(self.z_x_rotation.sin(), 0.0, self.z_x_rotation.cos())
            .wedge(&Vector3::new(0.0, 1.0, 0.0));

        CameraRender {
            // scale -> rotate z to x -> rotate y to xz -> translate
            view: Affine3::IDENTITY
                .translate(&(-self.position))
                .rotate(-self.y_zx_rotation, &front_y_plane)
                .rotate(-self.z_x_rotation, &BiVector3::new(0.0, 0.0, 1.0))
                .scale(&Scale3::new(2.0 / self.width, 2.0 / self.height, 1.0))
        }       
    }
}