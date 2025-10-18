use ash::ext::debug_utils;
use ash::khr::surface;
use ash::khr::swapchain;
use ash::{Device, Entry, Instance, vk};
use std::time::Instant;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
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
	pub surface: vk::SurfaceKHR,
	pub swapchain_ctx: SwapchainContext,
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

		VkSwap {
			surface,
			swapchain_ctx,
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
	pub fn draw_frame(&self) {}

	// have a cleanup in Core and Swap respectively.
	// Swap cleans every suspend signal, Core cleans on shutdown
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
