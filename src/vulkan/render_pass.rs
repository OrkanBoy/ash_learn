use ash::vk;

pub fn create_render_pass(
    device: &ash::Device,
    color_format: vk::Format,
) -> vk::RenderPass {
    let color_attachment_desc = vk::AttachmentDescription::builder()
        .format(color_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let subpass_desc = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .build();

    let info = vk::RenderPassCreateInfo::builder()
        .subpasses(&[subpass_desc])
        .attachments(&[color_attachment_desc])
        .build();

    unsafe {
        device.create_render_pass(&info, None)
            .expect("Failed to create render procedure(renderpass), setup color attachments and sub procedure(subpass) dependencies")
    }
}
