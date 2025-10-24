use super::{
	Device, DeviceContext, FrameData, InstanceContext, PipelineContext, SwapchainContext, Window,
	common::*, surface, vk,
};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct VkSwap {
	pub surface: vk::SurfaceKHR,
	pub swapchain_ctx: SwapchainContext,
	pub pipeline_ctx: PipelineContext,
	pub frames: Vec<FrameData>,
	pub cmd_pool: vk::CommandPool, // manages the memory used to store buffers
	pub current_frame: u32,
}
impl VkSwap {
	pub fn new(
		window: &Window,
		instance_ctx: &InstanceContext,
		device_ctx: &DeviceContext,
	) -> VkSwap {
		let surface = unsafe {
			ash_window::create_surface(
				instance_ctx.entry(),
				instance_ctx.instance(),
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.expect("Should have been able to create surface in create swapchain")
		};
		let swapchain_ctx = SwapchainContext::new(instance_ctx, device_ctx, window, surface);
		let pipeline_ctx = PipelineContext::new(&device_ctx, swapchain_ctx.swapchain_format);
		let cmd_pool = VkSwap::create_command_pool(device_ctx.device(), device_ctx.graphics_index);
		let mut frames: Vec<FrameData> = Vec::new();
		for _ in 0..FRAMES_IN_FLIGHT {
			frames.push(FrameData::new(device_ctx.device(), cmd_pool));
		}

		VkSwap {
			surface,
			swapchain_ctx,
			pipeline_ctx,
			frames,
			current_frame: 0,
			cmd_pool,
		}
	}

	pub fn record_command_buff(&self, img_idx: u32, device: &Device) {
		let cmd_buff = self
			.frames
			.get(self.current_frame as usize)
			.expect("current frame should be valid index into frames")
			.cmd_buff;
		let extent = self.swapchain_ctx.swapchain_extent;

		unsafe {
			device
				.begin_command_buffer(cmd_buff, &vk::CommandBufferBeginInfo::default())
				.expect("Should have been able to begin command_buffer")
		};
		// before starting to render, transfer swapchain image to COLOR_ATTACHMENT_OPTIMAL
		self.transition_img_layout(
			device,
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
				.swapchain_ctx
				.swapchain_img_views
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
				self.pipeline_ctx.graphics_pipeline,
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
			device,
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

	pub fn transition_img_layout(
		&self,
		device: &Device,
		img_idx: u32,
		old_layout: vk::ImageLayout,
		new_layout: vk::ImageLayout,
		src_access_mask: vk::AccessFlags2,
		dst_access_mask: vk::AccessFlags2,
		src_stage_mask: vk::PipelineStageFlags2,
		dst_stage_mask: vk::PipelineStageFlags2,
	) {
		let barrier = vk::ImageMemoryBarrier2 {
			src_stage_mask: src_stage_mask,
			src_access_mask: src_access_mask,
			dst_stage_mask: dst_stage_mask,
			dst_access_mask: dst_access_mask,
			old_layout: old_layout,
			new_layout: new_layout,
			src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
			dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
			image: *self
				.swapchain_ctx
				.swapchain_imgs
				.get(img_idx as usize)
				.expect("img_idx should always be valid for swapchain_imgs"),
			subresource_range: vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			},
			..Default::default()
		};
		let deps_info = vk::DependencyInfo {
			image_memory_barrier_count: 1,
			p_image_memory_barriers: &barrier,
			..Default::default()
		};

		let cmd_buff = self
			.frames
			.get(self.current_frame as usize)
			.expect("current frame should be valid index into frames")
			.cmd_buff;
		unsafe {
			device.cmd_pipeline_barrier2(cmd_buff, &deps_info);
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
	pub fn cleanup(&self, surface_loader: &surface::Instance, device: &Device) {
		// sync objects
		unsafe {
			for frame in &self.frames {
				device.destroy_semaphore(frame.img_available, None);
				device.destroy_semaphore(frame.render_finished, None);
				device.destroy_fence(frame.draw_fence, None);
			}
		}
		// views
		unsafe {
			self.swapchain_ctx
				.swapchain_img_views
				.iter()
				.for_each(|view| device.destroy_image_view(*view, None));
		}
		// swap (destroys imgs)
		unsafe {
			self.swapchain_ctx
				.swapchain_device
				.destroy_swapchain(self.swapchain_ctx.swapchain, None);
		}
		// pipeline + pipeline layout
		unsafe {
			device.destroy_pipeline_layout(self.pipeline_ctx.pipeline_layout, None);
			device.destroy_pipeline(self.pipeline_ctx.graphics_pipeline, None);
		}
		// cmd pool
		unsafe {
			device.destroy_command_pool(self.cmd_pool, None);
		}
		unsafe {
			surface_loader.destroy_surface(self.surface, None);
		}
	}
}
