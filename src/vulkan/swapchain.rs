use ash::{
    extensions::khr::{Surface, Swapchain},
    vk,
};

// we need multiple color images for the spwachian
// as we can have one image rendered to and one which is being presented
pub fn create_swapchain_khr(
    swapchain_extent: &mut vk::Extent2D,
    surface: &Surface,
    surface_khr: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    device: &ash::Device,
    swapchain: &Swapchain,
    physical_device: vk::PhysicalDevice,
    graphics_family_index: u32,
    present_family_index: u32,
) -> (
    vk::SwapchainKHR,
    Vec<vk::Image>,
    Vec<vk::ImageView>,
) {
    let capabilities = unsafe{surface.get_physical_device_surface_capabilities(physical_device, surface_khr).unwrap()};

    swapchain_extent.width = swapchain_extent.width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
    swapchain_extent.height = swapchain_extent.height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);

    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_khr)
        .min_image_count((capabilities.min_image_count + 1).min(capabilities.max_image_count))
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(*swapchain_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .image_sharing_mode(vk::SharingMode::CONCURRENT)
        .queue_family_indices(&[graphics_family_index, present_family_index])
        .build();

    let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None).unwrap() };
    let swapchain_images = unsafe { swapchain.get_swapchain_images(swapchain_khr).unwrap() };
    let swapchain_image_views = swapchain_images
        .iter()
        .map(|&image| {
            super::img::create_image_view(
                device, 
                image, 
                surface_format.format, 
                vk::ImageAspectFlags::COLOR
            )
        })
        .collect();

    (
        swapchain_khr,
        swapchain_images,
        swapchain_image_views,
    )
}

pub fn create_swapchain_framebuffers(
    device: &ash::Device,
    image_views: &[vk::ImageView],
    depth_image_view: vk::ImageView,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
) -> Vec<vk::Framebuffer> {
    image_views
        .iter()
        .map(|&image_view| {
            let info = vk::FramebufferCreateInfo::builder()
                .attachments(&[image_view, depth_image_view])
                .render_pass(render_pass)
                .width(extent.width)
                .height(extent.height)
                .layers(1)
                .build();

            unsafe { device.create_framebuffer(&info, None).unwrap() }
        })
        .collect()
}

pub fn choose_swapchain_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
        return vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        };
    }

    *formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_UNORM
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(&formats[0])
}

pub fn choose_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    let mut chosen = vk::PresentModeKHR::IMMEDIATE;
    for &mode in present_modes {
        if mode == vk::PresentModeKHR::MAILBOX {
            return mode;
        } else if mode == vk::PresentModeKHR::FIFO {
            chosen = mode;
        }
    }
    chosen
}