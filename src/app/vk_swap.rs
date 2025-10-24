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
