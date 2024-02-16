use std::{ffi::CString, io::Read, mem};

use ash::vk;
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

pub type Index = u32;

// TODO: write a proc macro which annotates the vertex struct and generate the attribute descriptor
const VERTEX_ATTRIB_DESCS: &[vk::VertexInputAttributeDescription] = &[
    vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: 0,
    },
    vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: core::mem::offset_of!(Vertex, color) as u32,
    },
    vk::VertexInputAttributeDescription {
        location: 2,
        binding: 0,
        format: vk::Format::R32G32_SFLOAT,
        offset: core::mem::offset_of!(Vertex, tex_coord) as u32,
    },
];

const BINDING_DESCS: &[vk::VertexInputBindingDescription] = &[
    vk::VertexInputBindingDescription {
        binding: 0,
        stride: mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }
];

pub fn new_pipeline_and_layout(
    device: &ash::Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    shader_compiler: &shaderc::Compiler,
    render_pass: vk::RenderPass,
    vertex_shader_path: &str,
    fragment_shader_path: &str,
) -> (vk::Pipeline, vk::PipelineLayout) {

    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&[
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
        ])
        .build();

    let vert_module = create_shader_module(
        device, 
        &shader_compiler, 
        vertex_shader_path,
        shaderc::ShaderKind::Vertex,
    );
    let frag_module = create_shader_module(
        device, 
        &shader_compiler, 
        fragment_shader_path,
        shaderc::ShaderKind::Fragment,
    );

    let entry_name = CString::new("main").unwrap();
    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(&entry_name)
        .build();
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(&entry_name)
        .build();

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(VERTEX_ATTRIB_DESCS)
        .vertex_binding_descriptions(BINDING_DESCS)
        .build();

    let input_assembly_create: vk::PipelineInputAssemblyStateCreateInfo = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();

    let viewport_create = vk::PipelineViewportStateCreateInfo::builder()
        .scissor_count(1)
        .viewport_count(1)
        .build();

    let rasterizer_create = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0)
        .build();

    let multisampling_create = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .min_sample_shading(1.0)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false)
        .build();

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ZERO)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build();
    let color_blend_attachments = [color_blend_attachment];

    let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::GREATER)
        .build();

    let layout = {
        let layout = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[
                descriptor_set_layout,
            ])
            .build();

        unsafe { device.create_pipeline_layout(&layout, None).unwrap() }
    };

    let info = vk::GraphicsPipelineCreateInfo::builder()
        .dynamic_state(&dynamic_state)
        .stages(&[vert_stage, frag_stage])
        .input_assembly_state(&input_assembly_create)
        .viewport_state(&viewport_create)
        .rasterization_state(&rasterizer_create)
        .multisample_state(&multisampling_create)
        .vertex_input_state(&vertex_input_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blending)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0) // what does this do?!
        .build();
    let pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
            .unwrap()[0]
    };

    unsafe {
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    };

    (pipeline, layout)
}

fn create_shader_module(
    device: &ash::Device, 
    shader_compiler: &shaderc::Compiler, 
    file_path: &str,
    shader_kind: shaderc::ShaderKind,
) -> vk::ShaderModule {
    let mut file = std::fs::File::open(file_path).unwrap();
    let mut source = String::new();
    file.read_to_string(&mut source).unwrap();

    let code = shader_compiler.compile_into_spirv(
        &source, 
        shader_kind, 
        file_path, 
        "main",
        None,
    ).unwrap().as_binary().to_vec();

    let info = vk::ShaderModuleCreateInfo::builder()
        .code(&code);
    unsafe {
        device
            .create_shader_module(&info, None)
            .unwrap()
    }
}