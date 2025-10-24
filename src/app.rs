use ash::ext::debug_utils;
use ash::khr::surface;
use ash::khr::swapchain;
use ash::{Device, Entry, Instance, vk};
use std::time::Instant;
use winit::window::Window;

pub mod common;
pub mod device_ctx;
pub mod frame_data;

pub mod instance_ctx;
pub mod pipeline_ctx;
pub mod swapchain_ctx;
pub mod vk_swap;
pub mod window;
pub use common::*;
pub use device_ctx::DeviceContext;
pub use frame_data::FrameData;
pub use instance_ctx::InstanceContext;
pub use pipeline_ctx::PipelineContext;
pub use swapchain_ctx::SwapchainContext;
pub use vk_swap::VkSwap;

pub struct VkCore {
	pub instance_ctx: InstanceContext,
	pub device_ctx: DeviceContext,
}
impl VkCore {
	pub fn new(window: &Window) -> VkCore {
		let instance_ctx = InstanceContext::new();
		let device_ctx = DeviceContext::new(&instance_ctx, window);

		VkCore {
			instance_ctx,
			device_ctx,
		}
	}
	pub fn cleanup(&self) {
		// debug messenger
		unsafe {
			self.instance_ctx
				.debug_utils_loader
				.destroy_debug_utils_messenger(self.instance_ctx.debug_messenger, None);
		}
		// device
		unsafe {
			self.device_ctx.device().destroy_device(None);
		}
		// instance
		unsafe {
			self.instance_ctx.instance().destroy_instance(None);
		}
	}
}

pub struct Application {
	pub window: Option<Window>,
	pub last_frame: Instant,

	vk: Option<VkCore>,
	vk_swap: Option<VkSwap>,
}

impl Application {
	pub fn new() -> Application {
		env_logger::builder()
			.filter_module("lvkrs", log::LevelFilter::Info)
			.format_timestamp(None)
			.init();

		log::info!("Building application!");
		Application {
			window: None,
			vk: None,
			vk_swap: None,

			last_frame: Instant::now(),
		}
	}
	pub fn vk(&self) -> &VkCore {
		self.vk
			.as_ref()
			.expect("VkCore should have been initialized")
	}
	pub fn vk_swap(&self) -> &VkSwap {
		self.vk_swap
			.as_ref()
			.expect("VkSwap should have been initialized")
	}

	// theoretical game update method
	pub fn update(&self, dt: f32) {}
	pub fn draw_frame(&self) {
		let device = self.vk().device_ctx.device();
		let curr_frame = self.vk_swap().current_frame;
		let frame = &self
			.vk_swap()
			.frames
			.get(curr_frame as usize)
			.expect("curr_frame should index into a valid frame");
		let swap_device = &self.vk_swap().swapchain_ctx.swapchain_device;
		let swapchain = self.vk_swap().swapchain_ctx.swapchain;
		let queue = self.vk().device_ctx.graphics_queue;
		unsafe {
			let _ = device.queue_wait_idle(queue);
		};

		let (img_idx, res) = unsafe {
			swap_device
				.acquire_next_image(swapchain, u64::MAX, frame.img_available, vk::Fence::null())
				.expect("Should have been able to acquire next image")
		};
		self.vk_swap().record_command_buff(img_idx, device);

		// reset fence
		match unsafe { device.reset_fences(&[frame.draw_fence]) } {
			Ok(()) => {}
			Err(e) => panic!("Failed to wait for fence: {:?}", e),
		};

		let wait_dest_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
		let submit_info = vk::SubmitInfo {
			p_wait_dst_stage_mask: &wait_dest_stage_mask,
			p_wait_semaphores: &frame.img_available,
			wait_semaphore_count: 1,
			p_command_buffers: &frame.cmd_buff,
			p_signal_semaphores: &frame.render_finished,
			signal_semaphore_count: 1,
			..Default::default()
		};
		match unsafe { device.queue_submit(queue, &[submit_info], frame.draw_fence) } {
			Ok(()) => {}
			Err(e) => panic!("Failed to submit to queue: {:?}", e),
		};

		// loop until done waiting for fence
		loop {
			match unsafe { device.wait_for_fences(&[frame.draw_fence], true, u64::MAX) } {
				Ok(()) => break,
				Err(vk::Result::TIMEOUT) => continue,
				Err(e) => panic!("Failed to wait for fence {:?}", e),
			}
		}

		let present_info_khr = vk::PresentInfoKHR {
			wait_semaphore_count: 1,
			p_wait_semaphores: &frame.render_finished,
			swapchain_count: 1,
			p_swapchains: &swapchain,
			p_image_indices: &img_idx,
			..Default::default()
		};
		let result = unsafe { swap_device.queue_present(queue, &present_info_khr) };
		match result {
			Ok(subobtimal) => {
				if subobtimal {
					log::info!("suboptimal swapchain");
				}
			}
			Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
				// window-resized
				// recreate_swapchain
			}
			Err(vk::Result::SUBOPTIMAL_KHR) => {}
			Err(e) => panic!("Failed to present: {:?}", e),
		};
	}
}
