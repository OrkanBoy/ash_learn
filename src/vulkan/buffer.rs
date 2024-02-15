use std::mem::size_of;

use ash::vk;

use super::device::find_memory_type_index;

pub fn create_buffer(
    device: &ash::Device,
    available_properties: &vk::PhysicalDeviceMemoryProperties, 
    required_properties: vk::MemoryPropertyFlags,
    usage: vk::BufferUsageFlags, 
    size: vk::DeviceSize,
) -> (vk::Buffer, vk::DeviceMemory) {
    unsafe {
        let buffer = device.create_buffer(
            &vk::BufferCreateInfo::builder()
                .usage(usage)
                .size(size)
                .sharing_mode(vk::SharingMode::EXCLUSIVE), 
            None,
        ).unwrap();

        let memory_requirements = device.get_buffer_memory_requirements(buffer);

        let memory_type_index = find_memory_type_index(
            memory_requirements.memory_type_bits,
            required_properties,
            &available_properties,
        );

        let memory = device.allocate_memory(
            &vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index)
                .build(), 
            None
        ).unwrap();

        device.bind_buffer_memory(buffer, memory, 0).unwrap();

        (buffer, memory)
    }
}