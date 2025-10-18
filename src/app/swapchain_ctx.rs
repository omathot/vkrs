use super::{swapchain, vk};

pub struct SwapchainContext {
	pub swapchain_device: swapchain::Device,
	pub swapchain: vk::SwapchainKHR,
	pub swapchain_format: vk::Format,
	pub swapchain_extent: vk::Extent2D,
	pub swapchain_imgs: Vec<vk::Image>,
	pub swapchain_img_views: Vec<vk::ImageView>,
}
