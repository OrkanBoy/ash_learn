use std::{ffi::c_void, mem::size_of, ptr::{null, null_mut}};

use ash::{extensions::{ext::DebugUtils, khr::{Surface, Swapchain}}, vk::{self, DebugUtilsMessengerEXT, Extent2D, SurfaceKHR}};

use self::device::QUEUE_FAMILY_INDICES;
use super::camera;
const FRAMES_IN_FLIGHT: u8 = 3;

pub mod init;
pub mod swapchain;
pub mod image;
pub mod device;
pub mod render_pass;
pub mod pipeline;
pub mod buffer;

pub struct Vulkan {
    instance: ash::Instance, 
    debug_utils: DebugUtils,
    debug_messenger: DebugUtilsMessengerEXT,

    surface: Surface,
    surface_khr: SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,

    device: ash::Device,
    physical_device: vk::PhysicalDevice,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    transient_command_pool: vk::CommandPool,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    graphics_family_index: u32,
    present_family_index: u32,

    swapchain: Swapchain,
    pub swapchain_extent: vk::Extent2D,
    swapchain_khr: vk::SwapchainKHR,
    swapchain_present_mode: vk::PresentModeKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    image_available_semaphores: [vk::Semaphore; FRAMES_IN_FLIGHT as usize],
    render_finished_semaphores: [vk::Semaphore; FRAMES_IN_FLIGHT as usize],
    in_flight_fences: [vk::Fence; FRAMES_IN_FLIGHT as usize],

    current_frame: usize,

    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,

    index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,

    camera_buffer: vk::Buffer,
    camera_memory: vk::DeviceMemory,
    camera_mapped_ptr: *mut c_void,
    camera_buffer_stride: vk::DeviceSize,

    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    descriptor_pool: vk::DescriptorPool,
}

impl Vulkan {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = ash::Entry::linked();
        let instance = init::create_instance(&entry); 
    
        let debug_utils = DebugUtils::new(&entry, &instance);
        let debug_messenger = init::create_messenger(&debug_utils);

        let surface = Surface::new(&entry, &instance);
        let surface_khr = unsafe { init::create_surface(
            &entry,
            &instance,
            window,
        )};

        // "fmi" means family index 
        let (physical_device, queue_family_indices) = device::get_physical_device_and_queue_family_indices(&instance, &surface, surface_khr);
        let [graphics_family_index, present_family_index] = queue_family_indices;
        let (device, [graphics_queue, present_queue]) = device::create_logical_device_and_queues(&instance, physical_device, &queue_family_indices);

        let command_pool = unsafe {    
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::builder()
                    .queue_family_index(graphics_family_index)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None
            ).unwrap()
        };
        let transient_command_pool = unsafe {    
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::builder()
                    .queue_family_index(graphics_family_index)
                    .flags(vk::CommandPoolCreateFlags::TRANSIENT),
                None
            ).unwrap()
        };
        
        let command_buffers = unsafe {    
            device.allocate_command_buffers(&
                vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(FRAMES_IN_FLIGHT as u32)
            ).unwrap()
        };

        let swapchain = Swapchain::new(&instance, &device);
        
        let surface_format = swapchain::choose_swapchain_format(unsafe{&surface
            .get_physical_device_surface_formats(physical_device, surface_khr)
            .unwrap()
        });

        let size = window.inner_size();
        let mut swapchain_extent = Extent2D {
            width: size.width,
            height: size.height,
        };

        let swapchain_present_mode = swapchain::choose_swapchain_present_mode(&unsafe { surface.get_physical_device_surface_present_modes(physical_device, surface_khr).unwrap() });

        let (
            swapchain_khr, 
            swapchain_images, 
            swapchain_image_views, 
        ) = swapchain::create_swapchain_khr(
            &mut swapchain_extent,
            &surface,
            surface_khr,
            surface_format,
            swapchain_present_mode,
            &device,
            &swapchain,
            physical_device,
            graphics_family_index,
            present_family_index,
        );

        use pipeline::{Vertex, Index};
        let vertices = &[
            Vertex {
                position: [-1.0, 1.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];
        let indices: &[Index] = &[0, 1, 2];

        let physical_device_memory_properties = unsafe{instance.get_physical_device_memory_properties(physical_device)};

        let vertex_buffer_size = (vertices.len() * size_of::<Vertex>()) as vk::DeviceSize;
        let index_buffer_size = (indices.len() * size_of::<Index>()) as vk::DeviceSize;
        let (vertex_buffer, vertex_memory) = buffer::create_buffer(
            &device, 
            &physical_device_memory_properties, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vertex_buffer_size,
        );

        let (index_buffer, index_memory) = buffer::create_buffer(
            &device, 
            &physical_device_memory_properties, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            index_buffer_size,
        );

        let min_uniform_buffer_offset_alignment = unsafe{instance.get_physical_device_properties(physical_device)}.limits.min_uniform_buffer_offset_alignment;
        let camera_buffer_stride = min_uniform_buffer_offset_alignment.max(size_of::<camera::CameraRender>() as vk::DeviceSize);
        let camera_buffer_size = camera_buffer_stride * FRAMES_IN_FLIGHT as vk::DeviceSize;
        let (camera_buffer, camera_memory) = buffer::create_buffer(
            &device, 
            &physical_device_memory_properties, 
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            camera_buffer_size,
        );
        let camera_mapped_ptr = unsafe{device.map_memory(camera_memory, 0, camera_buffer_size, vk::MemoryMapFlags::empty()).unwrap() as *mut c_void};

        unsafe {
            let (staging_buffer, staging_memory) = buffer::create_buffer(
                &device, 
                &physical_device_memory_properties, 
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vertex_buffer_size + index_buffer_size,
            );

            {
                let ptr = device.map_memory(staging_memory, 0, vertex_buffer_size, vk::MemoryMapFlags::empty()).unwrap();
                (ptr as *mut Vertex).copy_from(vertices.as_ptr(), vertices.len());
                (ptr.add(vertex_buffer_size as usize) as *mut Index).copy_from(indices.as_ptr(), indices.len());
                device.unmap_memory(staging_memory);
            }

            let command_buffer = command_buffers[0];

            device.begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default()).unwrap();

            device.cmd_copy_buffer(
                command_buffer, 
                staging_buffer, 
                vertex_buffer, 
                &[vk::BufferCopy{
                    src_offset: 0,
                    dst_offset: 0,
                    size: vertex_buffer_size,
                }]
            );

            device.cmd_copy_buffer(
                command_buffer, 
                staging_buffer, 
                index_buffer, 
                &[vk::BufferCopy{
                    src_offset: vertex_buffer_size,
                    dst_offset: 0,
                    size: index_buffer_size,
                }]
            );

            device.end_command_buffer(command_buffer).unwrap();

            device.queue_submit(
                graphics_queue, 
                &[vk::SubmitInfo::builder()
                    .command_buffers(&[command_buffer])
                    .build()], 
                vk::Fence::null()
            ).unwrap();

            device.queue_wait_idle(graphics_queue).unwrap();

            device.free_memory(staging_memory, None);
            device.destroy_buffer(staging_buffer, None);    
        }

        let descriptor_set_layout = unsafe {device.create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .build()
            ]), 
            None,
        ).unwrap()};

        let descriptor_pool = unsafe{device.create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::builder()
                .max_sets(1)
                .pool_sizes(&[
                    vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                        descriptor_count: 1,
                    }
                ])
            , None
        ).unwrap()};

        let descriptor_set = unsafe{device.allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&[descriptor_set_layout])
                .build(),
        ).unwrap()[0]};

        unsafe {
            device.update_descriptor_sets(
                &[
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_set)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                        .dst_binding(0)
                        .dst_array_element(0)
                        .buffer_info(&[
                            vk::DescriptorBufferInfo {
                                buffer: camera_buffer,
                                offset: 0,
                                // do not pass size of whole camera buffer 
                                range: camera_buffer_stride,
                            }
                        ])
                        .build()
                ], 
                &[],
            );
        }
        
        let render_pass = render_pass::create_render_pass(&device, surface_format.format);
        
        let swapchain_framebuffers = swapchain::create_swapchain_framebuffers(&device, &swapchain_image_views, render_pass, swapchain_extent);

        let shader_compiler = shaderc::Compiler::new().unwrap();

        let (pipeline, pipeline_layout) = pipeline::new_pipeline_and_layout(
            &device, 
            descriptor_set_layout,
            &shader_compiler, 
            render_pass, "C:/users/snick/dev/ash_learn/src/shaders/main.vert", 
            "C:/users/snick/dev/ash_learn/src/shaders/main.frag"
        );

        let mut image_available_semaphores = [Default::default(); FRAMES_IN_FLIGHT as usize];
        let mut render_finished_semaphores = [Default::default(); FRAMES_IN_FLIGHT as usize];
        let mut in_flight_fences = [Default::default(); FRAMES_IN_FLIGHT as usize];

        let semaphore_info = &vk::SemaphoreCreateInfo::builder();
        let fence_info = &vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        for frame in 0..FRAMES_IN_FLIGHT as usize {
            unsafe {
                image_available_semaphores[frame] = device.create_semaphore(&semaphore_info, None).unwrap();
                render_finished_semaphores[frame] = device.create_semaphore(&semaphore_info, None).unwrap();
                in_flight_fences[frame] = device.create_fence(&fence_info, None).unwrap();
            }
        }

        Vulkan {
            instance,

            debug_utils,
            debug_messenger,

            surface,
            surface_khr,
            surface_format,

            physical_device,

            device,

            command_pool,
            command_buffers,
            transient_command_pool,

            graphics_queue,
            present_queue,

            graphics_family_index,
            present_family_index,


            swapchain,
            swapchain_khr,
            swapchain_extent,
            swapchain_present_mode,
            swapchain_images,
            swapchain_image_views,
            swapchain_framebuffers,

            render_pass,
            pipeline,
            pipeline_layout,
            current_frame: 0,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,

            vertex_buffer,
            vertex_memory,

            index_buffer,
            index_memory,
            
            camera_buffer,
            camera_memory,
            camera_mapped_ptr,
            camera_buffer_stride,

            descriptor_set_layout,
            descriptor_pool,
            descriptor_set,
        }
    }

    pub fn update_camera(&mut self, camera: &camera::Camera) {
        unsafe {
            let offset = self.current_frame * self.camera_buffer_stride as usize;
            (self.camera_mapped_ptr.add(offset) as *mut camera::CameraRender).write(camera.to_render());
        }
    }

    pub fn renew_swapchain(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            for i in 0..self.swapchain_image_views.len() {
                self.device.destroy_framebuffer(self.swapchain_framebuffers[i], None);
                self.device.destroy_image_view(self.swapchain_image_views[i], None);   
            }
            self.swapchain.destroy_swapchain(self.swapchain_khr, None);

            (
                self.swapchain_khr,
                self.swapchain_images,
                self.swapchain_image_views,
            ) = swapchain::create_swapchain_khr(
                &mut self.swapchain_extent,
                &self.surface,
                self.surface_khr,
                self.surface_format,
                self.swapchain_present_mode,
                &self.device,
                &self.swapchain,
                self.physical_device,
                self.graphics_family_index,
                self.present_family_index,
            );
    
            self.swapchain_framebuffers = swapchain::create_swapchain_framebuffers(
                &self.device, 
                &self.swapchain_image_views, 
                self.render_pass, 
                self.swapchain_extent,
            );
        }
    }

    pub fn draw_frame(&mut self) {
        unsafe {
            let image_available_semaphore = self.image_available_semaphores[self.current_frame];
            let render_finished_semaphore = self.render_finished_semaphores[self.current_frame];
            let in_flight_fence = self.in_flight_fences[self.current_frame];
            let command_buffer = self.command_buffers[self.current_frame];
    
            let fences = &[in_flight_fence];
            self.device.wait_for_fences(fences, true, u64::MAX).unwrap();
            self.device.reset_fences(fences).unwrap();
    
            let image_index = match self.swapchain.acquire_next_image(
                self.swapchain_khr, 
                u64::MAX, 
                image_available_semaphore, 
                vk::Fence::null(),
            ) {
                Ok((image_index, _)) => image_index,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return,
                Err(err) => panic!("Error acquiring image: {}", err),
            };
        
            self.device.reset_command_buffer(
                command_buffer, 
                vk::CommandBufferResetFlags::empty()
            ).expect("Failed to reset command buffer contents"); 
    
            // record command buffer
            {   
                let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                    .render_pass(self.render_pass)
                    .framebuffer(self.swapchain_framebuffers[image_index as usize])
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D {
                            x: 0, y: 0,
                        },
                        extent: self.swapchain_extent,
                    })
                    .clear_values(&[
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.2, 1.0],
                            }
                        },
                    ]);
                        
                self.device.begin_command_buffer(
                    command_buffer, 
                    &vk::CommandBufferBeginInfo::default()
                ).expect("Failed to begin recording command buffer");
    
                self.device.cmd_begin_render_pass(
                    command_buffer, 
                    &render_pass_begin_info, 
                    vk::SubpassContents::INLINE
                );

                self.device.cmd_bind_pipeline(
                    command_buffer, 
                    vk::PipelineBindPoint::GRAPHICS, 
                    self.pipeline
                );
    
                self.device.cmd_set_viewport(
                    command_buffer, 
                    0, 
                    &[vk::Viewport {
                        x: 0.0, 
                        y: 0.0,
                        width: self.swapchain_extent.width as f32, 
                        height: self.swapchain_extent.height as f32,
                        min_depth: 0.0, 
                        max_depth: 1.0, 
                    }]
                );
                self.device.cmd_set_scissor(
                    command_buffer, 
                    0, 
                    &[vk::Rect2D {
                        offset: vk::Offset2D {
                            x: 0,
                            y: 0,
                        },
                        extent: self.swapchain_extent,
                    }]
                );

                self.device.cmd_bind_descriptor_sets(
                    command_buffer, 
                    vk::PipelineBindPoint::GRAPHICS, 
                    self.pipeline_layout, 
                    0, 
                    &[self.descriptor_set], 
                    &[self.camera_buffer_stride as u32 * self.current_frame as u32]
                );
                
                self.device.cmd_bind_index_buffer(
                    command_buffer, 
                    self.index_buffer, 
                    0, 
                    vk::IndexType::UINT32
                );
                self.device.cmd_bind_vertex_buffers(
                    command_buffer, 
                    0, 
                    &[self.vertex_buffer], &[0]);

                self.device.cmd_draw_indexed(command_buffer, 3, 1, 0, 0, 0);

                self.device.cmd_end_render_pass(command_buffer);
    
                self.device.end_command_buffer(command_buffer).expect("Could not end recording command buffer");
            }

            // render
            {
                let render_info = vk::SubmitInfo::builder()
                    .command_buffers(&[command_buffer])
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .wait_semaphores(&[image_available_semaphore])
                    .signal_semaphores(&[render_finished_semaphore])
                    .build();
                let render_infos = [render_info];
    
                self.device.queue_submit(self.graphics_queue, &render_infos, in_flight_fence).unwrap();
            }

            //present
            {
                let present_info = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&[render_finished_semaphore])
                    .swapchains(&[self.swapchain_khr])
                    .image_indices(&[image_index])
                    .build();
                match self.swapchain.queue_present(self.present_queue, &present_info) {
                    Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return,
                    Err(err) => panic!("Error presenting: {}", err),
                    _ => {},
                }
            }
    
            self.current_frame = (self.current_frame + 1) % FRAMES_IN_FLIGHT as usize;
        }
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);

            self.device.unmap_memory(self.camera_memory);
            self.camera_mapped_ptr = null_mut();
            self.device.free_memory(self.camera_memory, None);
            self.device.destroy_buffer(self.camera_buffer, None);  

            self.device.free_memory(self.index_memory, None);
            self.device.destroy_buffer(self.index_buffer, None);            

            self.device.free_memory(self.vertex_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);

            for frame in 0..FRAMES_IN_FLIGHT as usize {
                self.device.destroy_semaphore(self.image_available_semaphores[frame], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[frame], None);
                self.device.destroy_fence(self.in_flight_fences[frame], None);
            }

            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);

            for i in 0..self.swapchain_image_views.len() {
                self.device.destroy_framebuffer(self.swapchain_framebuffers[i], None);
                self.device.destroy_image_view(self.swapchain_image_views[i], None);   
            }
            self.swapchain.destroy_swapchain(self.swapchain_khr, None);

            self.device.destroy_render_pass(self.render_pass, None);

            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_command_pool(self.transient_command_pool, None);

            self.device.destroy_device(None);

            self.surface.destroy_surface(self.surface_khr, None);

            self.debug_utils.destroy_debug_utils_messenger(self.debug_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}