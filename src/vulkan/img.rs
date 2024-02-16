use ash::vk;

pub fn create_image(
    device: &ash::Device,
    physical_device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    width: u32,
    height: u32,
    usage: vk::ImageUsageFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    memory_properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(vk::SampleCountFlags::TYPE_1)
        .flags(vk::ImageCreateFlags::empty());

    let image = unsafe { device.create_image(&info, None).unwrap() };

    let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
    let mem_type_index = super::device::find_memory_type_index(
        mem_requirements.memory_type_bits,
        memory_properties,
        &physical_device_memory_properties,
    );

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type_index)
        .build();
    let memory = unsafe {
        let mem = device.allocate_memory(&alloc_info, None).unwrap();
        device.bind_image_memory(image, mem, 0).unwrap();
        mem
    };

    (image, memory)
}

pub fn create_image_view(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
) -> vk::ImageView {
    let create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    
    unsafe { device.create_image_view(&create_info, None).unwrap() }
}