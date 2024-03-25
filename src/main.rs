use std::time::{Duration, Instant};
mod input;
mod vulkan;
mod math;
mod camera;
mod instance;

pub const TARGET_FPS: u16 = 60; 
pub const TARGET_DT: f32 = 1.0 / TARGET_FPS as f32;

use ash::vk;
use math::Vector3;
use winit::{dpi::PhysicalSize, event::{Event, KeyEvent, WindowEvent}, event_loop::{ControlFlow, EventLoop}, *};
fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = window::WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 800,
            height: 600,
        })
        .build(&event_loop)
        .unwrap();

    let mut vulkan = vulkan::Vulkan::new(&window);

    let mut input_state = input::InputState::new();

    let instant = Instant::now();
    let mut time = 0.0;
    let mut camera = camera::Camera::new(
        Vector3::new(0.0, 0.0, -1.0),
        3.0,
        3.0 * vulkan.swapchain_extent.height as f32 / vulkan.swapchain_extent.width as f32,
        0.1,
        2.0,
        2.0,
    );

    let _ = event_loop.run(|event, elwt| {
        use winit::keyboard::*;

        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(PhysicalSize{width, height}) => {
                    camera.height = camera.width * height as f32 / width as f32;

                    vulkan.swapchain_extent = vk::Extent2D {
                        width,
                        height,
                    };

                    if width != 0 && height != 0 {
                        vulkan.renew_swapchain();
                    }
                }
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                    ..
                } => {
                    if key_code == KeyCode::Escape {
                        elwt.exit();
                    } else {
                        input_state.set_key_pressed(key_code, state.is_pressed());
                    }
                },
                _ => {}
            }
            Event::AboutToWait if vulkan.swapchain_extent.width != 0 && vulkan.swapchain_extent.height != 0 => {
                vulkan.update_camera(&camera);
                vulkan.draw_frame();

                // framerate
                {
                    let new_time = instant.elapsed().as_secs_f32();
                    let dt = new_time - time;
                    if dt < TARGET_DT {
                        std::thread::sleep(Duration::from_secs_f32(TARGET_DT - dt));
                    }
                    time += TARGET_DT;
                }

                // game logic
                {
                    use winit::keyboard::KeyCode::*;
                    let w = input_state.is_key_pressed(KeyW);
                    let a = input_state.is_key_pressed(KeyA);
                    let s = input_state.is_key_pressed(KeyS);
                    let d = input_state.is_key_pressed(KeyD);

                    camera.update();

                    let d_translation = camera.translation_speed * TARGET_DT;

                    if w && !s {
                        camera.position.x += d_translation * camera.front_x;
                        camera.position.z += d_translation * camera.front_z;
                    } else if !w && s {
                        camera.position.x -= d_translation * camera.front_x;
                        camera.position.z -= d_translation * camera.front_z;
                    }

                    if d && !a {
                        camera.position.z -= d_translation * camera.front_x;
                        camera.position.x += d_translation * camera.front_z;
                    } else if !d && a {
                        camera.position.z += d_translation * camera.front_x;
                        camera.position.x -= d_translation * camera.front_z;
                    }
                    
                    let up = input_state.is_key_pressed(ArrowUp);
                    let down = input_state.is_key_pressed(ArrowDown);
                    let right = input_state.is_key_pressed(ArrowRight);
                    let left = input_state.is_key_pressed(ArrowLeft);

                    let d_rotation = TARGET_DT * camera.rotation_speed;
                    if right && !left {
                        camera.z_x_rotation += d_rotation;
                    } else if !right && left {
                        camera.z_x_rotation -= d_rotation;
                    }

                    if up && !down {
                        camera.zx_y_rotation -= d_rotation;
                    } else if !up && down {
                        camera.zx_y_rotation += d_rotation;
                    }
                }
            }
            _ => ()
        }
    });
}