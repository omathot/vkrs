use ash::ext::debug_utils;
use ash::khr::surface;
use ash::khr::swapchain;
use ash::{Device, Entry, Instance, vk};
use std::time::Instant;
use winit::window::Window;

pub mod common;
pub mod device_ctx;
pub mod init;
pub mod instance_ctx;
pub mod pipeline_ctx;
pub mod swapchain_ctx;
pub mod utils;
pub mod window;
pub use common::*;
pub use device_ctx::DeviceContext;
pub use instance_ctx::InstanceContext;
pub use pipeline_ctx::PipelineContext;
pub use swapchain_ctx::SwapchainContext;

pub struct VkCore {
	pub instance_ctx: InstanceContext,
	pub device_ctx: DeviceContext,
	pub pipeline_ctx: PipelineContext,
}
impl VkCore {
	pub fn new(window: &Window) -> VkCore {
		let instance_ctx = InstanceContext::new();
		let device_ctx = DeviceContext::new(&instance_ctx, window);
		let pipeline_ctx = PipelineContext::new(&device_ctx);

		VkCore {
			instance_ctx,
			device_ctx,
			pipeline_ctx,
		}
	}
}

pub struct VkSwap {
	pub surface: Option<vk::SurfaceKHR>,
	pub swapchain_ctx: SwapchainContext,
}
impl VkSwap {
	pub fn new() {}
}

pub struct Application {
	pub window: Option<Window>,

	vk: Option<VkCore>,
	vk_swapchain: Option<VkSwap>,
	pub last_frame: Instant,

	pub surface: Option<vk::SurfaceKHR>,

	pub swapchain_device: Option<swapchain::Device>,
	pub swapchain: Option<vk::SwapchainKHR>,
	pub swapchain_format: Option<vk::Format>,
	pub swapchain_extent: Option<vk::Extent2D>,
	pub swapchain_imgs: Option<Vec<vk::Image>>,
	pub swapchain_img_views: Option<Vec<vk::ImageView>>,
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
			vk_swapchain: None,

			last_frame: Instant::now(),

			surface: None,
			swapchain_device: None,
			swapchain: None,
			swapchain_format: None,
			swapchain_extent: None,
			swapchain_imgs: None,
			swapchain_img_views: Some(Vec::new()), // not good
		}
	}
	pub fn vk(&self) -> &VkCore {
		self.vk
			.as_ref()
			.expect("VkCore should have been initialized")
	}
	pub fn vk_swap(&self) -> &VkSwap {
		self.vk_swapchain
			.as_ref()
			.expect("VkSwap should have been initialized")
	}
	pub fn update(&self, dt: f32) {}
	pub fn draw_frame(&self) {}
	pub fn cleanup(&mut self) {
		// if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
		// 	unsafe { loader.destroy_debug_utils_messenger(messenger, None) };
		// }
		// if let (Some(surface), Some(loader)) = (self.surface, &self.surface_loader) {
		// 	unsafe { loader.destroy_surface(surface, None) };
		// }
		// if let (Some(swapchain), Some(swap_device)) = (self.swapchain, &self.swapchain_device) {
		// 	unsafe { swap_device.destroy_swapchain(swapchain, None) };
		// }
		// if let Some(device) = &self.device {
		// 	if let Some(images) = &self.swapchain_imgs {
		// 		unsafe {
		// 			images
		// 				.iter()
		// 				.for_each(|img| device.destroy_image(*img, None));
		// 		}
		// 	}
		// 	unsafe { device.destroy_device(None) };
		// }
		// if let Some(instance) = &self.instance {
		// 	unsafe { instance.destroy_instance(None) };
		// }
	}
}
