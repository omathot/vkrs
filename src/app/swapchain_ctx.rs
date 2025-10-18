use super::{Device, DeviceContext, InstanceContext, Window, swapchain, vk};

pub struct SwapchainContext {
	pub swapchain_device: swapchain::Device,
	pub swapchain: vk::SwapchainKHR,
	pub swapchain_format: vk::Format,
	pub swapchain_extent: vk::Extent2D,
	pub swapchain_imgs: Vec<vk::Image>,
	pub swapchain_img_views: Vec<vk::ImageView>,
}
impl SwapchainContext {
	pub fn new(
		instance_ctx: &InstanceContext,
		device_ctx: &DeviceContext,
		window: &Window,
		surface: vk::SurfaceKHR,
	) -> SwapchainContext {
		let (swapchain_device, swapchain, swapchain_format, swapchain_extent, swapchain_imgs) =
			SwapchainContext::create_swapchain(instance_ctx, device_ctx, window, surface);
		let swapchain_img_views = SwapchainContext::create_image_views(
			&swapchain_imgs,
			swapchain_format,
			device_ctx.device(),
		);

		SwapchainContext {
			swapchain_device: swapchain_device,
			swapchain: swapchain,
			swapchain_format: swapchain_format,
			swapchain_extent: swapchain_extent,
			swapchain_imgs: swapchain_imgs,
			swapchain_img_views: swapchain_img_views,
		}
	}

	fn create_swapchain(
		instance_ctx: &InstanceContext,
		device_ctx: &DeviceContext,
		window: &Window,
		surface: vk::SurfaceKHR,
	) -> (
		swapchain::Device,
		vk::SwapchainKHR,
		vk::Format,
		vk::Extent2D,
		Vec<vk::Image>,
	) {
		let loader = instance_ctx.surface_loader();
		let phys_device = device_ctx.phys_device();
		// It is important that we only try to query for swap chain support after verifying that the extension is available.
		let capabilities = unsafe {
			loader
				.get_physical_device_surface_capabilities(phys_device, surface)
				.expect("Should be able to query surface capabilities")
		};
		let formats = unsafe {
			loader
				.get_physical_device_surface_formats(phys_device, surface)
				.expect("Should be able to query surface formats")
		};
		let present_modes = unsafe {
			loader
				.get_physical_device_surface_present_modes(phys_device, surface)
				.expect("Should be able to query surface present modes")
		};

		let format = SwapchainContext::choose_swap_format(&formats);
		// TODO: Make this work with the assumed format used when constructing VkCore
		let swapchain_format = format.format;
		let swapchain_extent = SwapchainContext::choose_swap_extent(&capabilities, window);
		let present_mode = SwapchainContext::choose_swap_present_mode(&present_modes);
		let mut img_count = capabilities.min_image_count + 1;
		if capabilities.max_image_count > 0 && img_count > capabilities.max_image_count {
			img_count = capabilities.max_image_count;
		}

		let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
			flags: vk::SwapchainCreateFlagsKHR::empty(),
			surface: surface,
			min_image_count: img_count,
			image_format: format.format,
			image_color_space: format.color_space,
			image_extent: swapchain_extent,
			image_array_layers: 1, // always 1 unless doing stereostopic 3d app
			image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
			image_sharing_mode: vk::SharingMode::EXCLUSIVE, // assume same family for present and graphics
			pre_transform: capabilities.current_transform,
			composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
			present_mode: present_mode,
			clipped: vk::TRUE,
			..Default::default()
		};
		let indices: [u32; 2];
		if device_ctx.graphics_index != device_ctx.present_index {
			let graphics_idx = device_ctx.graphics_index;
			let present_idx = device_ctx.present_index;
			indices = [graphics_idx, present_idx];
			swapchain_create_info = swapchain_create_info
				.image_sharing_mode(vk::SharingMode::CONCURRENT)
				.queue_family_indices(&indices);
			swapchain_create_info.queue_family_index_count = 2;
		}

		let instance = instance_ctx.instance();
		let swapchain_device = swapchain::Device::new(instance, device_ctx.device());
		let swapchain = unsafe {
			swapchain_device
				.create_swapchain(&swapchain_create_info, None)
				.expect("Should have been able to create swapchain")
		};
		let swapchain_imgs = unsafe {
			swapchain_device
				.get_swapchain_images(swapchain)
				.expect("Should have been able to get swapchain images")
		};

		// this is kinda nasty, could return in a struct but meh. hopefully some more logical separation
		// of construction will make itself obvious
		(
			swapchain_device,
			swapchain,
			swapchain_format,
			swapchain_extent,
			swapchain_imgs,
		)
	}

	fn choose_swap_format(formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
		formats
			.iter()
			.find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
			.copied()
			.unwrap_or(formats[0])
	}
	fn choose_swap_present_mode(present_modes: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
		present_modes
			.iter()
			.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX) // need to deref mode
			.copied()
			.unwrap_or(vk::PresentModeKHR::FIFO)
	}
	fn choose_swap_extent(
		capabilities: &vk::SurfaceCapabilitiesKHR,
		window: &Window,
	) -> vk::Extent2D {
		if capabilities.current_extent.width != u32::MAX {
			return capabilities.current_extent;
		}
		let size = window.inner_size();
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

	fn create_image_views(
		swapchain_imgs: &Vec<vk::Image>,
		swapchain_format: vk::Format,
		device: &Device,
	) -> Vec<vk::ImageView> {
		let img_views: Vec<vk::ImageView> = swapchain_imgs
			.iter()
			.map(|&img| {
				let view_create_info = vk::ImageViewCreateInfo {
					image: img,
					view_type: vk::ImageViewType::TYPE_2D,
					format: swapchain_format,
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
					device
						.create_image_view(&view_create_info, None)
						.expect("Should have been able to create image view")
				}
			})
			.collect();

		img_views
	}
}
