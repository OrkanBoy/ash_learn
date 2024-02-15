use std::time::{Duration, Instant};
mod input;
mod vulkan;
mod math;
mod camera;

pub const TARGET_FPS: u16 = 5; 
pub const TARGET_DT: f32 = 1.0 / TARGET_FPS as f32;

use ash::{extensions::ext::DebugUtils, vk::{self, DebugUtilsMessengerEXT}};
use winit::{dpi::PhysicalSize, event::{Event, KeyEvent, WindowEvent}, event_loop::{ControlFlow, EventLoop}, *};
fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = window::WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 500,
            height: 500,
        })
        .build(&event_loop).unwrap();

    let mut vulkan = vulkan::Vulkan::new(&window);

    let mut input_state = input::InputState::new();

    let instant = Instant::now();
    let mut time = 0.0;

    let _ = event_loop.run(|event, elwt| {
        use winit::keyboard::*;

        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(PhysicalSize{width, height}) => {
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


                // user-input_state
                {
                    input_state.previous_keys_pressed_bitmask = input_state.keys_pressed_bitmask;
                }
            }
            _ => ()
        }
    });
}