use ash::vk;

pub fn create_render_pass(
    device: &ash::Device,
    color_format: vk::Format,
    depth_format: vk::Format,
) -> vk::RenderPass {
    let color_attachment_desc = vk::AttachmentDescription::builder()
        .format(color_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();

    let depth_attachment_desc = vk::AttachmentDescription::builder()
        .format(depth_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let subpass_0 = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .depth_stencil_attachment(&depth_attachment_ref)
        .build();

    let dep_ext_0 = vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask:
            vk::PipelineStageFlags::ALL_COMMANDS,
        dst_stage_mask:
            vk::PipelineStageFlags::ALL_COMMANDS,
        src_access_mask:
            vk::AccessFlags::COLOR_ATTACHMENT_READ |
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE |
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | 
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dst_access_mask: 
            vk::AccessFlags::COLOR_ATTACHMENT_READ | 
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE |
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | 
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::empty(),
    };

    let dep_0_ext = vk::SubpassDependency {
        src_subpass: 0,
        dst_subpass: vk::SUBPASS_EXTERNAL,
        src_stage_mask:
            vk::PipelineStageFlags::ALL_COMMANDS,
        dst_stage_mask:
            vk::PipelineStageFlags::ALL_COMMANDS,
        src_access_mask:
            vk::AccessFlags::COLOR_ATTACHMENT_READ |
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE |
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | 
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dst_access_mask: 
            vk::AccessFlags::COLOR_ATTACHMENT_READ | 
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE |
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | 
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::empty(),
    };


    let info = vk::RenderPassCreateInfo::builder()
        .subpasses(&[subpass_0])
        .attachments(&[color_attachment_desc, depth_attachment_desc])
        .dependencies(&[dep_ext_0, dep_0_ext])
        .build();

    unsafe {
        device.create_render_pass(&info, None).unwrap()
    }
}
