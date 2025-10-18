use ash::ext::debug_utils;
use ash::khr::surface;
use ash::khr::swapchain;
use ash::{Device, Entry, Instance, vk};
use std::time::Instant;
use winit::window::Window;

pub mod common;
pub mod device_ctx;
pub mod helpers;
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
	pub fn new(window: &Window) {
		let instance_ctx = InstanceContext::new();
		let device_ctx = DeviceContext::new(&instance_ctx, window);
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

	pub physical_device: Option<vk::PhysicalDevice>,
	pub device: Option<Device>, // logical connection - 'i am running vk on this physical device'
	pub graphics_index: u32,
	pub present_index: u32,
	pub graphics_queue: Option<vk::Queue>,

	pub surface_loader: Option<surface::Instance>,
	pub surface: Option<vk::SurfaceKHR>,

	pub swapchain_device: Option<swapchain::Device>,
	pub swapchain: Option<vk::SwapchainKHR>,
	pub swapchain_format: Option<vk::Format>,
	pub swapchain_extent: Option<vk::Extent2D>,
	pub swapchain_imgs: Option<Vec<vk::Image>>,
	pub swapchain_img_views: Option<Vec<vk::ImageView>>,

	pub pipeline_layout: Option<vk::PipelineLayout>,
	pub graphics_pipeline: Option<vk::Pipeline>,

	pub command_pool: Option<vk::CommandPool>, // manages the memory used to store buffers
	pub command_buff: Option<vk::CommandBuffer>,

	pub last_frame: Instant,
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

			physical_device: None,
			device: None,
			graphics_index: 0,
			present_index: 0,
			graphics_queue: None,
			surface_loader: None,
			surface: None,
			swapchain_device: None,
			swapchain: None,
			swapchain_format: None,
			swapchain_extent: None,
			swapchain_imgs: None,
			swapchain_img_views: Some(Vec::new()), // not good
			pipeline_layout: None,
			graphics_pipeline: None,
			command_pool: None,
			command_buff: None,
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
