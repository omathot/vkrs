use super::{Device, DeviceContext, vk};
use std::ffi::c_void;

pub struct PipelineContext {
	pub pipeline_layout: vk::PipelineLayout,
	pub graphics_pipeline: vk::Pipeline,

	pub command_pool: vk::CommandPool, // manages the memory used to store buffers
	pub command_buff: vk::CommandBuffer,
}

impl PipelineContext {
	pub fn new(device_ctx: &DeviceContext, swap_format: vk::Format) -> PipelineContext {
		let (pipeline_layout, graphics_pipeline) =
			PipelineContext::create_graphics_pipeline(device_ctx.device(), swap_format);
		let command_pool =
			PipelineContext::create_command_pool(device_ctx.device(), device_ctx.graphics_index);
		let command_buff = PipelineContext::create_command_buff(device_ctx.device(), command_pool);

		PipelineContext {
			pipeline_layout,
			graphics_pipeline,
			command_pool,
			command_buff,
		}
	}

	fn create_graphics_pipeline(
		device: &Device,
		swap_format: vk::Format,
	) -> (vk::PipelineLayout, vk::Pipeline) {
		// TODO: normalize path
		let shader_code = include_bytes!("../../shaders/slang.spv");
		debug_assert!(shader_code.len() > 0, "shader_code byte len <= 0");
		let shader_module = PipelineContext::create_shader_module(device, shader_code);
		let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
			stage: vk::ShaderStageFlags::VERTEX,
			module: shader_module,
			p_name: c"vertMain".as_ptr(),
			..Default::default()
		};
		let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
			stage: vk::ShaderStageFlags::FRAGMENT,
			module: shader_module,
			p_name: c"fragMain".as_ptr(),
			..Default::default()
		};
		let shader_stages = [vert_shader_stage_info, frag_shader_stage_info];

		let dyn_states = [vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
		let dyn_state_info = vk::PipelineDynamicStateCreateInfo {
			dynamic_state_count: dyn_states.len() as u32,
			p_dynamic_states: dyn_states.as_ptr(),
			..Default::default()
		};
		let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
			vertex_binding_description_count: 0,
			vertex_attribute_description_count: 0,
			..Default::default()
		};
		let input_asm_info = vk::PipelineInputAssemblyStateCreateInfo {
			topology: vk::PrimitiveTopology::TRIANGLE_LIST,
			..Default::default()
		};
		let viewport_info = vk::PipelineViewportStateCreateInfo {
			viewport_count: 1,
			scissor_count: 1,
			..Default::default()
		};
		let rasterizer_info = vk::PipelineRasterizationStateCreateInfo {
			depth_clamp_enable: vk::FALSE,
			rasterizer_discard_enable: vk::FALSE,
			polygon_mode: vk::PolygonMode::FILL,
			cull_mode: vk::CullModeFlags::BACK,
			front_face: vk::FrontFace::CLOCKWISE,
			depth_bias_enable: vk::FALSE,
			depth_bias_slope_factor: 1.,
			line_width: 1.,
			..Default::default()
		};
		let multisampling_info = vk::PipelineMultisampleStateCreateInfo {
			rasterization_samples: vk::SampleCountFlags::TYPE_1,
			sample_shading_enable: vk::FALSE,
			..Default::default()
		};
		let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
			blend_enable: vk::TRUE,
			color_write_mask: vk::ColorComponentFlags::R
				| vk::ColorComponentFlags::G
				| vk::ColorComponentFlags::B
				| vk::ColorComponentFlags::A,
			src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
			dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
			color_blend_op: vk::BlendOp::ADD,
			src_alpha_blend_factor: vk::BlendFactor::ONE,
			dst_alpha_blend_factor: vk::BlendFactor::ZERO,
			alpha_blend_op: vk::BlendOp::ADD,
			..Default::default()
		};
		let color_blend_info = vk::PipelineColorBlendStateCreateInfo {
			logic_op_enable: vk::FALSE,
			logic_op: vk::LogicOp::COPY,
			attachment_count: 1,
			p_attachments: &color_blend_attachment,
			..Default::default()
		};
		let pipeline_info = vk::PipelineLayoutCreateInfo {
			set_layout_count: 0,
			push_constant_range_count: 0,
			..Default::default()
		};

		let pipeline_layout = unsafe {
			device
				.create_pipeline_layout(&pipeline_info, None)
				.expect("Should have been able to create pipeline layout")
		};
		let pipeline_rendering_info = vk::PipelineRenderingCreateInfo {
			color_attachment_count: 1,
			p_color_attachment_formats: &swap_format,
			..Default::default()
		};
		let pipeline_info = vk::GraphicsPipelineCreateInfo {
			p_next: &pipeline_rendering_info as *const _ as *const c_void, // cast to raw ptr (cursed)
			stage_count: 2,
			p_stages: shader_stages.as_ptr(),
			p_vertex_input_state: &vertex_input_info,
			p_input_assembly_state: &input_asm_info,
			p_viewport_state: &viewport_info,
			p_rasterization_state: &rasterizer_info,
			p_multisample_state: &multisampling_info,
			p_color_blend_state: &color_blend_info,
			p_dynamic_state: &dyn_state_info,
			layout: pipeline_layout,
			render_pass: vk::RenderPass::null(),
			..Default::default()
		};
		let graphics_pipeline = unsafe {
			device
				.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
				.expect("Should have been able to greate graphics pipeline")[0] // only creating one for now
		};

		(pipeline_layout, graphics_pipeline)
	}

	pub fn create_shader_module(device: &Device, code: &'static [u8]) -> vk::ShaderModule {
		let create_info = vk::ShaderModuleCreateInfo {
			code_size: code.len(),
			p_code: code.as_ptr() as *const u32,
			..Default::default()
		};
		unsafe {
			device
				.create_shader_module(&create_info, None)
				.expect("Should have been able to create shader module")
		}
	}

	fn create_command_pool(device: &Device, graphics_idx: u32) -> vk::CommandPool {
		let pool_info = vk::CommandPoolCreateInfo {
			flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
			queue_family_index: graphics_idx,
			..Default::default()
		};
		let command_pool = unsafe {
			device
				.create_command_pool(&pool_info, None)
				.expect("Should have been able to create command pool")
		};

		command_pool
	}
	fn create_command_buff(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
		let alloc_info = vk::CommandBufferAllocateInfo {
			command_pool: command_pool,
			level: vk::CommandBufferLevel::PRIMARY,
			command_buffer_count: 1,
			..Default::default()
		};
		let command_buff = unsafe {
			device
				.allocate_command_buffers(&alloc_info)
				.expect("Should have been able to allocate command buff")[0] // only creating one for now
		};

		command_buff
	}
}
