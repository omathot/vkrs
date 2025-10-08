use super::{Application, vk};
use crate::common::*;
use ash::khr::{surface, swapchain};
use std::ffi::{CString, c_char};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

impl Application {
	pub fn init_vulkan(&mut self) {
		self.create_instance();
		self.setup_debug_messenger();
		self.create_surface();
		self.pick_physical_device();
		self.create_logical_device();
		self.find_queue_families();
		self.create_swap_chain();
		self.create_image_views();
	}

	fn create_instance(&mut self) {
		let required_layers = self.get_required_layers();
		let required_extensions: Vec<*const c_char> = self.get_required_extensions();

		let app_info = vk::ApplicationInfo {
			application_version: vk::make_api_version(0, 1, 0, 0),
			api_version: vk::make_api_version(0, 1, 4, 0),
			..Default::default()
		};
		let create_info = vk::InstanceCreateInfo {
			p_application_info: &app_info,
			enabled_layer_count: required_layers.len() as u32,
			pp_enabled_layer_names: required_layers.as_ptr(),
			enabled_extension_count: required_extensions.len() as u32,
			pp_enabled_extension_names: required_extensions.as_ptr(),
			..Default::default()
		};

		let instance = unsafe {
			self.entry
				.create_instance(&create_info, None)
				.expect("Should have been able to create instance")
		};
		self.instance = Some(instance);
	}

	fn create_surface(&mut self) {
		let entry = &self.entry;
		let instance = self.instance.as_ref().unwrap();
		let window = self.window.as_ref().unwrap();
		let surface = unsafe {
			ash_window::create_surface(
				entry,
				instance,
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.expect("Should have been able to create surface")
		};

		self.surface_loader = Some(surface::Instance::new(entry, instance));
		self.surface = Some(surface);
	}

	fn get_required_extensions(&self) -> Vec<*const c_char> {
		let mut extension_names = WL_REQUIRED_EXTENSIONS.to_vec();
		if ENABLE_VALIDATION_LAYERS {
			extension_names.push(vk::EXT_DEBUG_UTILS_NAME);
		}
		log::info!("{} required extensions:", extension_names.len());
		extension_names
			.iter()
			.for_each(|extension| log::info!("\t{:?}", extension));
		println!("");
		extension_names.iter().map(|cstr| cstr.as_ptr()).collect()
	}

	fn get_required_layers(&self) -> CStringArray {
		// query layers
		let layer_properties = unsafe {
			self.entry
				.enumerate_instance_layer_properties()
				.expect("Should have been able to get layer properties from entry")
		};
		log::info!(
			"{} available instance layer properties:",
			layer_properties.len()
		);
		layer_properties
			.iter()
			.for_each(|property| log::info!("\t{:?}", property.layer_name_as_c_str().unwrap()));
		println!("");

		// make required layers
		let mut required_layers: Vec<&str> = Vec::new();
		if ENABLE_VALIDATION_LAYERS {
			required_layers.extend(VALIDATION_LAYERS.iter());
		}
		if required_layers.iter().any(|required_layer| {
			let cstr_name = CString::new(*required_layer)
				.expect("Should have been able to create CString from required layer name");
			!layer_properties
				.iter()
				.any(|property| property.layer_name_as_c_str().unwrap() == cstr_name.as_c_str())
		}) {
			panic!("One or more required layers are not supported!");
		}
		let required_layer_names: Vec<CString> = required_layers
			.iter()
			.map(|layer| CString::new(*layer).unwrap())
			.collect();
		let required_layer_names_ptrs: Vec<*const c_char> = required_layer_names
			.iter()
			.map(|layer| layer.as_ptr())
			.collect();
		CStringArray::new(required_layer_names, required_layer_names_ptrs)
	}

	fn pick_physical_device(&mut self) {
		let instance = self
			.instance
			.as_ref()
			.expect("Instance should be init before physical device");
		let devices = unsafe {
			instance
				.enumerate_physical_devices()
				.expect("Should be able to list physical devices")
		};
		if devices.is_empty() {
			panic!("No vk physical devices to use");
		}
		let mut candidates: Vec<(u32, vk::PhysicalDevice, String)> = Vec::new();
		for device in devices.into_iter() {
			let features = unsafe { instance.get_physical_device_features(device) };
			let properties = unsafe { instance.get_physical_device_properties(device) };
			let name = properties
				.device_name_as_c_str()
				.unwrap()
				.to_str()
				.unwrap()
				.to_owned();
			let mut score = 0;
			log::info!("Checking device {:?}", name);
			if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
				score += 1000;
			}
			// requirements
			if features.geometry_shader != vk::TRUE {
				log::warn!("device {} does not have a geometry shader", name);
				continue;
			}
			if !self.has_minimum_queue_families_reqs(device) {
				log::warn!("Device {} has no suitable queue families, skipping", name);
				continue;
			}

			score += properties.limits.max_image_dimension2_d;
			if features.geometry_shader != vk::TRUE {
				log::warn!("device {:?} has no geometry shader", name);
				continue;
			}
			candidates.push((score, device, name));
		}
		if candidates.is_empty() {
			panic!("Could not find suitable GPU");
		}
		candidates.sort_by(|a, b| b.0.cmp(&a.0));
		let &(score, device, ref name) = &candidates[0];
		log::info!("picked device {}: score = {}", name, score);
		self.physical_device = Some(device);
	}

	fn create_logical_device(&mut self) {
		let instance = self.instance.as_ref().unwrap();
		let loader = self.surface_loader.as_ref().unwrap();
		let phys_device = self.physical_device.unwrap();

		// queue
		let prio: f32 = 0.;
		let device_queue_create_info = vk::DeviceQueueCreateInfo {
			queue_family_index: self.graphics_index,
			p_queue_priorities: &prio,
			queue_count: 1,
			..Default::default()
		};
		//features
		let mut dynamic_rendering = vk::PhysicalDeviceDynamicRenderingFeatures {
			dynamic_rendering: vk::TRUE,
			..Default::default()
		};
		let mut extended_dynamic_state = vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT {
			extended_dynamic_state: vk::TRUE,
			..Default::default()
		};
		// device extensions
		let device_extensions: Vec<*const c_char> = DEVICE_REQUIRED_EXTENSIONS
			.iter()
			.map(|ext| ext.as_ptr())
			.collect();
		let device_create_info = vk::DeviceCreateInfo {
			p_queue_create_infos: &device_queue_create_info,
			queue_create_info_count: 1,
			enabled_extension_count: DEVICE_REQUIRED_EXTENSIONS.len() as u32,
			pp_enabled_extension_names: device_extensions.as_ptr(),
			..Default::default()
		}
		.push_next(&mut dynamic_rendering)
		.push_next(&mut extended_dynamic_state);

		self.device = unsafe {
			Some(
				instance
					.create_device(phys_device, &device_create_info, None)
					.expect("Should have been able to create logical device"),
			)
		};
	}

	fn choose_swap_format(&self, formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
		formats
			.iter()
			.find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
			.copied()
			.unwrap_or(formats[0])
	}

	fn choose_swap_present_mode(
		&self,
		present_modes: &Vec<vk::PresentModeKHR>,
	) -> vk::PresentModeKHR {
		present_modes
			.iter()
			.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX) // need to deref mode
			.copied()
			.unwrap_or(vk::PresentModeKHR::FIFO)
	}

	fn choose_swap_extent(&self, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
		if capabilities.current_extent.width != u32::MAX {
			return capabilities.current_extent;
		}
		let size = self.window.as_ref().unwrap().inner_size();
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

	fn find_queue_families(&mut self) {
		let instance = self.instance.as_ref().unwrap();
		let queue_family_properties = unsafe {
			instance.get_physical_device_queue_family_properties(self.physical_device.unwrap())
		};

		let supports_present = |index: usize| -> bool {
			unsafe {
				let loader = self.surface_loader.as_ref().unwrap();
				loader
					.get_physical_device_surface_support(
						self.physical_device.unwrap(),
						index as u32,
						self.surface.unwrap(),
					)
					.unwrap_or(false)
			}
		};

		let mut graphics_idx = queue_family_properties
			.iter()
			.position(|properties| properties.queue_flags.contains(vk::QueueFlags::GRAPHICS))
			.expect("Should have been able to find a graphics queue family property");
		let present_idx = if supports_present(graphics_idx) {
			graphics_idx
		} else {
			// try to find single family that supports both
			queue_family_properties
				.iter()
				.enumerate()
				.position(|(idx, properties)| {
					properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
						&& supports_present(idx)
				})
				.map(|idx| {
					graphics_idx = idx;
					idx
				})
				.or_else(|| {
					// use separate family for present
					queue_family_properties
						.iter()
						.enumerate()
						.find(|(idx, _)| supports_present(*idx))
						.map(|(idx, _)| idx)
				})
				.expect("Should have been able to find graphics and present queues")
		};
		self.graphics_index = graphics_idx as u32;
		self.present_index = present_idx as u32;
		if let Some(device) = &self.device {
			self.graphics_queue = unsafe { Some(device.get_device_queue(self.graphics_index, 0)) };
		}
		log::info!(
			"graphics index: [{}], present index: [{}]",
			self.graphics_index,
			self.present_index
		);
	}

	fn create_swap_chain(&mut self) {
		let loader = self.surface_loader.as_ref().unwrap();
		let phys_device = self.physical_device.unwrap();
		let surface = self.surface.unwrap();
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

		let format = self.choose_swap_format(&formats);
		self.swap_chain_format = Some(format.format);
		self.swap_chain_extent = Some(self.choose_swap_extent(&capabilities));
		let present_mode = self.choose_swap_present_mode(&present_modes);
		let mut img_count = capabilities.min_image_count + 1;
		if capabilities.max_image_count > 0 && img_count > capabilities.max_image_count {
			img_count = capabilities.max_image_count;
		}

		let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
			flags: vk::SwapchainCreateFlagsKHR::empty(),
			surface: self.surface.unwrap(),
			min_image_count: img_count,
			image_format: format.format,
			image_color_space: format.color_space,
			image_extent: self.swap_chain_extent.unwrap(),
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
		if self.graphics_index != self.present_index {
			let graphics_idx = self.graphics_index;
			let present_idx = self.present_index;
			indices = [graphics_idx, present_idx];
			swapchain_create_info = swapchain_create_info
				.image_sharing_mode(vk::SharingMode::CONCURRENT)
				.queue_family_indices(&indices);
			swapchain_create_info.queue_family_index_count = 2;
		}

		let instance = self.instance.as_ref().unwrap();
		self.swap_chain_device = Some(swapchain::Device::new(
			instance,
			self.device.as_ref().unwrap(),
		));
		let swap_device = self.swap_chain_device.as_ref().unwrap();
		self.swap_chain = unsafe {
			Some(
				swap_device
					.create_swapchain(&swapchain_create_info, None)
					.expect("Should have been able to create swapchain"),
			)
		};
		self.swap_chain_imgs = unsafe {
			Some(
				swap_device
					.get_swapchain_images(self.swap_chain.unwrap())
					.expect("Should have been able to get swapchain images"),
			)
		};
	}
	fn create_image_views(&mut self) {
		// now gets initialized in app new, but still need to make that better
		// self.swap_chain_img_views = Some(Vec::new()); // never gets intialized before, temp fix

		self.swap_chain_img_views.as_mut().unwrap().clear();
		let img_views: Vec<vk::ImageView> = self
			.swap_chain_imgs
			.as_ref()
			.unwrap()
			.iter()
			.map(|&img| {
				let view_create_info = vk::ImageViewCreateInfo {
					image: img,
					view_type: vk::ImageViewType::TYPE_2D,
					format: self.swap_chain_format.unwrap(),
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
					self.device
						.as_ref()
						.unwrap()
						.create_image_view(&view_create_info, None)
						.expect("Should have been able to create image view")
				}
			})
			.collect();
		self.swap_chain_img_views = Some(img_views);
	}
	fn create_graphics_pipeline(&mut self) {
		//
	}
}
