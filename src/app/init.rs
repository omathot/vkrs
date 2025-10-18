use super::{Application, vk};
use crate::common::*;
use ash::khr::{surface, swapchain};
use std::{
	any::Any,
	ffi::{CString, c_char, c_void},
};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

impl Application {
	pub fn init_vulkan(&mut self) {
		// self.create_swapchain();
		self.create_image_views();
		self.create_graphics_pipeline();
		self.create_command_pool();
		self.create_command_buff();
	}

	fn choose_swap_format(&self, formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
		formats
			.iter()
			.find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
			.copied()
			.unwrap_or(formats[0])
	}

	fn choose_swap_present_mode(
		&self,
		present_modes: &Vec<vk::PresentModeKHR>,
	) -> vk::PresentModeKHR {
		present_modes
			.iter()
			.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX) // need to deref mode
			.copied()
			.unwrap_or(vk::PresentModeKHR::FIFO)
	}

	fn choose_swap_extent(&self, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
		if capabilities.current_extent.width != u32::MAX {
			return capabilities.current_extent;
		}
		let size = self.window.as_ref().unwrap().inner_size();
		vk::Extent2D {
			width: size.width.clamp(
				capabilities.min_image_extent.width,
				capabilities.max_image_extent.width,
			),
			height: size.width.clamp(
				capabilities.min_image_extent.height,
				capabilities.max_image_extent.height,
			),
		}
	}

	// fn create_swapchain(&mut self) {
	// 	let loader = self.surface_loader.as_ref().unwrap();
	// 	let phys_device = self.physical_device.unwrap();
	// 	let surface = self.surface.unwrap();
	// 	// It is important that we only try to query for swap chain support after verifying that the extension is available.
	// 	let capabilities = unsafe {
	// 		loader
	// 			.get_physical_device_surface_capabilities(phys_device, surface)
	// 			.expect("Should be able to query surface capabilities")
	// 	};
	// 	let formats = unsafe {
	// 		loader
	// 			.get_physical_device_surface_formats(phys_device, surface)
	// 			.expect("Should be able to query surface formats")
	// 	};
	// 	let present_modes = unsafe {
	// 		loader
	// 			.get_physical_device_surface_present_modes(phys_device, surface)
	// 			.expect("Should be able to query surface present modes")
	// 	};

	// 	let format = self.choose_swap_format(&formats);
	// 	self.swapchain_format = Some(format.format);
	// 	self.swapchain_extent = Some(self.choose_swap_extent(&capabilities));
	// 	let present_mode = self.choose_swap_present_mode(&present_modes);
	// 	let mut img_count = capabilities.min_image_count + 1;
	// 	if capabilities.max_image_count > 0 && img_count > capabilities.max_image_count {
	// 		img_count = capabilities.max_image_count;
	// 	}

	// 	let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
	// 		flags: vk::SwapchainCreateFlagsKHR::empty(),
	// 		surface: self.surface.unwrap(),
	// 		min_image_count: img_count,
	// 		image_format: format.format,
	// 		image_color_space: format.color_space,
	// 		image_extent: self.swapchain_extent.unwrap(),
	// 		image_array_layers: 1, // always 1 unless doing stereostopic 3d app
	// 		image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
	// 		image_sharing_mode: vk::SharingMode::EXCLUSIVE, // assume same family for present and graphics
	// 		pre_transform: capabilities.current_transform,
	// 		composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
	// 		present_mode: present_mode,
	// 		clipped: vk::TRUE,
	// 		..Default::default()
	// 	};
	// 	let indices: [u32; 2];
	// 	if self.graphics_index != self.present_index {
	// 		let graphics_idx = self.graphics_index;
	// 		let present_idx = self.present_index;
	// 		indices = [graphics_idx, present_idx];
	// 		swapchain_create_info = swapchain_create_info
	// 			.image_sharing_mode(vk::SharingMode::CONCURRENT)
	// 			.queue_family_indices(&indices);
	// 		swapchain_create_info.queue_family_index_count = 2;
	// 	}

	// 	let instance = self.instance.as_ref().unwrap();
	// 	self.swapchain_device = Some(swapchain::Device::new(
	// 		instance,
	// 		self.device.as_ref().unwrap(),
	// 	));
	// 	let swap_device = self.swapchain_device.as_ref().unwrap();
	// 	self.swapchain = unsafe {
	// 		Some(
	// 			swap_device
	// 				.create_swapchain(&swapchain_create_info, None)
	// 				.expect("Should have been able to create swapchain"),
	// 		)
	// 	};
	// 	self.swapchain_imgs = unsafe {
	// 		Some(
	// 			swap_device
	// 				.get_swapchain_images(self.swapchain.unwrap())
	// 				.expect("Should have been able to get swapchain images"),
	// 		)
	// 	};
	// }
	fn create_image_views(&mut self) {
		// now gets initialized in app new, but still need to make that better
		// self.swapchain_img_views = Some(Vec::new()); // never gets intialized before, temp fix

		self.swapchain_img_views.as_mut().unwrap().clear();
		let img_views: Vec<vk::ImageView> = self
			.swapchain_imgs
			.as_ref()
			.unwrap()
			.iter()
			.map(|&img| {
				let view_create_info = vk::ImageViewCreateInfo {
					image: img,
					view_type: vk::ImageViewType::TYPE_2D,
					format: self.swapchain_format.unwrap(),
					subresource_range: vk::ImageSubresourceRange {
						aspect_mask: vk::ImageAspectFlags::COLOR,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					},
					components: vk::ComponentMapping::default(),
					..Default::default()
				};
				unsafe {
					self.device
						.as_ref()
						.unwrap()
						.create_image_view(&view_create_info, None)
						.expect("Should have been able to create image view")
				}
			})
			.collect();
		self.swapchain_img_views = Some(img_views);
	}
	fn create_graphics_pipeline(&mut self) {
		// TODO: normalize path
		let shader_code = include_bytes!("../../shaders/slang.spv");
		debug_assert!(shader_code.len() > 0, "shader_code byte len <= 0");
		let shader_module = self.create_shader_module(shader_code);
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

		self.pipeline_layout = unsafe {
			Some(
				self.device
					.as_ref()
					.unwrap()
					.create_pipeline_layout(&pipeline_info, None)
					.expect("Should have been able to create pipeline layout"),
			)
		};
		let pipeline_rendering_info = vk::PipelineRenderingCreateInfo {
			color_attachment_count: 1,
			p_color_attachment_formats: self.swapchain_format.as_ref().unwrap(),
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
			layout: self.pipeline_layout.unwrap(),
			render_pass: vk::RenderPass::null(),
			..Default::default()
		};
		self.graphics_pipeline = unsafe {
			Some(
				self.device
					.as_ref()
					.unwrap()
					.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
					.expect("Should have been able to greate graphics pipeline")[0], // only creating one for now
			)
		};
	}
	fn create_command_pool(&mut self) {
		let pool_info = vk::CommandPoolCreateInfo {
			flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
			queue_family_index: self.graphics_index,
			..Default::default()
		};
		self.command_pool = unsafe {
			Some(
				self.device
					.as_ref()
					.unwrap()
					.create_command_pool(&pool_info, None)
					.expect("Should have been able to create command pool"),
			)
		};
	}
	fn create_command_buff(&mut self) {
		let alloc_info = vk::CommandBufferAllocateInfo {
			command_pool: self.command_pool.unwrap(),
			level: vk::CommandBufferLevel::PRIMARY,
			command_buffer_count: 1,
			..Default::default()
		};
		self.command_buff = unsafe {
			Some(
				self.device
					.as_ref()
					.unwrap()
					.allocate_command_buffers(&alloc_info)
					.expect("Should have been able to allocate command buff")[0], // only creating one for now
			)
		};
	}
	fn record_command_buff(&self, img_idx: u32) {
		let cmd_buff = self.command_buff.unwrap();
		let device = self.device.as_ref().unwrap();
		let extent = self.swapchain_extent.unwrap();

		unsafe {
			device
				.begin_command_buffer(cmd_buff, &vk::CommandBufferBeginInfo::default())
				.expect("Should have been able to begin command_buffer")
		};
		// before starting to render, transfer swapchain image to COLOR_ATTACHMENT_OPTIMAL
		self.transition_img_layout(
			img_idx,
			vk::ImageLayout::UNDEFINED,
			vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
			vk::AccessFlags2::empty(), // src access mask (no need to wait for previous op)
			vk::AccessFlags2::COLOR_ATTACHMENT_WRITE, // dst access mask
			vk::PipelineStageFlags2::TOP_OF_PIPE, // src stage
			vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, // dst stage
		);
		// vk::ClearValue is a union expression, can only hold one field
		let clear_color = vk::ClearValue {
			color: vk::ClearColorValue {
				float32: [0., 0., 0., 1.],
			},
		};
		let attachment_info = vk::RenderingAttachmentInfo {
			image_view: *self
				.swapchain_img_views
				.as_ref()
				.unwrap()
				.get(img_idx as usize)
				.expect("img_idx should always be valid for swapchain img views"),
			image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
			load_op: vk::AttachmentLoadOp::CLEAR,
			store_op: vk::AttachmentStoreOp::STORE,
			clear_value: clear_color,
			..Default::default()
		};
		let render_info = vk::RenderingInfo {
			render_area: vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent: extent,
			},
			layer_count: 1,
			color_attachment_count: 1,
			p_color_attachments: &attachment_info,
			..Default::default()
		};

		unsafe {
			// begin rendering
			device.cmd_begin_rendering(cmd_buff, &render_info);
			// bind pipeline
			device.cmd_bind_pipeline(
				cmd_buff,
				vk::PipelineBindPoint::GRAPHICS,
				self.graphics_pipeline.unwrap(),
			);
			// set dynamic states
			device.cmd_set_viewport(
				cmd_buff,
				0,
				&[vk::Viewport {
					x: 0.,
					y: 0.,
					width: extent.width as f32,
					height: extent.height as f32,
					min_depth: 0.,
					max_depth: 1.,
				}],
			);
			device.cmd_set_scissor(
				cmd_buff,
				0,
				&[vk::Rect2D {
					offset: vk::Offset2D { x: 0, y: 0 },
					extent: extent,
				}],
			);

			device.cmd_draw(cmd_buff, 3, 1, 0, 0);
			device.cmd_end_rendering(cmd_buff);
		};
		// transition back to present to screen
		self.transition_img_layout(
			img_idx,
			vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
			vk::ImageLayout::PRESENT_SRC_KHR,
			vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
			vk::AccessFlags2::empty(),
			vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
			vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
		);
		unsafe {
			device
				.end_command_buffer(cmd_buff)
				.expect("Should have been able to end cmd buff")
		};
	}
}
