use ash::ext::debug_utils;
use ash::khr::surface;
use ash::khr::swapchain;
use ash::{Device, Entry, Instance, vk};
use std::time::Instant;
use winit::window::Window;

pub mod common;
pub mod device_ctx;
pub mod frame_data;
pub mod init;
pub mod instance_ctx;
pub mod pipeline_ctx;
pub mod swapchain_ctx;
pub mod utils;
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
		let curr_frame = self.vk_swap().current_frame;
		let frame = &self
			.vk_swap()
			.frames
			.get(curr_frame as usize)
			.expect("curr_frame should index into a valid frame");
		let swap_device = &self.vk_swap().swapchain_ctx.swapchain_device;
		let swapchain = self.vk_swap().swapchain_ctx.swapchain;

		// let (img_idx, res) = unsafe {
		// 	swap_device
		// 		.acquire_next_image(swapchain, u64::MAX, frame.img_available, vk::Fence::null())
		// 		.expect("Should have been able to acquire next image")
		// };
	}
}
