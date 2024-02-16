use std::{ffi::c_char, mem};

use ash::{extensions::khr::{Surface, Swapchain}, vk};

const DEVICE_EXTENSION_NAMES: &[*const c_char] = &[
    Swapchain::name().as_ptr(),
];

pub fn get_physical_device_and_queue_family_indices(
    instance: &ash::Instance,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, [u32; QUEUE_FAMILY_INDICES]) {
    let physical_device = unsafe { instance.enumerate_physical_devices() }.unwrap()[0];

    let queue_family_props = unsafe {instance.get_physical_device_queue_family_properties(physical_device)};

    const INVALID_INDEX: u32 = u32::MAX;
    let mut graphics = INVALID_INDEX;
    let mut present = INVALID_INDEX;

    for (index, family_props) in queue_family_props.iter().filter(|p| p.queue_count > 0).enumerate() {
        let index = index as u32;

        if family_props.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics == INVALID_INDEX {
            graphics = index;
        }

        let present_support = unsafe {
            surface.get_physical_device_surface_support(physical_device, index, surface_khr)
        }.unwrap();
        if present_support && (present == INVALID_INDEX || (graphics != INVALID_INDEX && graphics == present))
        {
            present = index;
        }
    }

    (
        physical_device,
        [graphics, present],
    )
}

pub const QUEUE_FAMILY_INDICES: usize = 2; 
pub fn create_logical_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_indices: &[u32; QUEUE_FAMILY_INDICES],
) -> (ash::Device, [vk::Queue; QUEUE_FAMILY_INDICES]) {

    let mut queue_infos = [unsafe { mem::zeroed() }; QUEUE_FAMILY_INDICES];
    for i in 0..QUEUE_FAMILY_INDICES {
        queue_infos[i] = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_indices[i])
            .queue_priorities(&[1.0])
            .build()
    }

    let enabled_featues = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .build();

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_features(&enabled_featues)
        .enabled_extension_names(DEVICE_EXTENSION_NAMES);

    unsafe {
        let device = instance
            .create_device(physical_device, &info, None)
            .unwrap();
        let mut queues = [mem::zeroed(); QUEUE_FAMILY_INDICES];

        for i in 0..QUEUE_FAMILY_INDICES {
            queues[i] = device.get_device_queue(queue_family_indices[i], 0);
        }

        (device, queues)
    }
}


pub fn find_memory_type_index(
    supported_types: u32,
    required_properties: vk::MemoryPropertyFlags,
    available_properties: &vk::PhysicalDeviceMemoryProperties,
) -> u32 {
    for i in 0..available_properties.memory_type_count {
        if supported_types & (1 << i) != 0
            && available_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return i;
        }
    }
    panic!("Could not find suitable memory type");
}

pub fn find_depth_format(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    formats: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    unsafe { 
        match tiling {
            vk::ImageTiling::LINEAR => {
                for &f in formats.iter() {
                    let props = instance.get_physical_device_format_properties(physical_device, f);
                    if features & props.linear_tiling_features == features {
                        return f;
                    }
                }
            }
            vk::ImageTiling::OPTIMAL => {
                for &f in formats.iter() {
                    let props = instance.get_physical_device_format_properties(physical_device, f);
                    if features & props.optimal_tiling_features == features {
                        return f;
                    }
                }
            }
            _ => {},
        } 
        panic!()
    }
}