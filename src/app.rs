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

pub struct VkSwap {
	pub surface: vk::SurfaceKHR,
	pub swapchain_ctx: SwapchainContext,
	pub pipeline_ctx: PipelineContext,
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

		VkSwap {
			surface,
			swapchain_ctx,
			pipeline_ctx,
		}
	}
	pub fn cleanup(&self, surface_loader: &surface::Instance, device: &Device) {
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
			device.destroy_command_pool(self.pipeline_ctx.command_pool, None);
		}
		unsafe {
			surface_loader.destroy_surface(self.surface, None);
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
}
