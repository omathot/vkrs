use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::{Device, Instance, InstanceContext, Window, common::*, surface, vk};
use std::ffi::{c_char, c_void};

pub struct DeviceContext {
	pub physical_device: vk::PhysicalDevice,
	pub device: Device, // logical connection - 'i am running vk on this physical device'

	pub graphics_index: u32,
	pub present_index: u32,
	pub graphics_queue: vk::Queue,
}

impl DeviceContext {
	pub fn new(instance_ctx: &InstanceContext, window: &Window) -> DeviceContext {
		// tmp surface for device creation
		let tmp_surface = unsafe {
			ash_window::create_surface(
				instance_ctx.entry(),
				instance_ctx.instance(),
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.expect("Should have been able to make tmp surface for device creation")
		};

		let physical_device = DeviceContext::pick_physical_device(instance_ctx, &tmp_surface);
		let (graphics_idx, present_idx) =
			DeviceContext::find_queue_families(instance_ctx, &physical_device, &tmp_surface);

		let device = DeviceContext::create_logical_device(
			instance_ctx.instance(),
			physical_device,
			graphics_idx,
		);
		let graphics_queue = unsafe { device.get_device_queue(graphics_idx, 0) };

		unsafe {
			instance_ctx
				.surface_loader()
				.destroy_surface(tmp_surface, None);
		}

		DeviceContext {
			physical_device,
			device,
			graphics_index: graphics_idx,
			present_index: present_idx,
			graphics_queue,
		}
	}

	fn pick_physical_device(
		instance_ctx: &InstanceContext,
		tmp_surface: &vk::SurfaceKHR,
	) -> vk::PhysicalDevice {
		let instance = instance_ctx.instance();
		let surface_loader = instance_ctx.surface_loader();

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
			// query device features
			let mut vk12_features = vk::PhysicalDeviceVulkan12Features::default();
			let mut vk11_features = vk::PhysicalDeviceVulkan11Features {
				p_next: &mut vk12_features as *const _ as *mut c_void,
				..Default::default()
			};
			let mut features2 = vk::PhysicalDeviceFeatures2 {
				p_next: &mut vk11_features as *const _ as *mut c_void,
				..Default::default()
			};
			unsafe {
				instance.get_physical_device_features2(device, &mut features2);
			}

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
			if features2.features.geometry_shader != vk::TRUE {
				log::warn!("device {} does not have a geometry shader", name);
				continue;
			}
			if features2.features.geometry_shader != vk::TRUE {
				log::warn!("device {:?} has no geometry shader", name);
				continue;
			}
			if vk11_features.shader_draw_parameters == vk::FALSE {
				log::warn!("Device {} does not support shaderDrawParameters", name);
				continue;
			}
			if vk12_features.buffer_device_address != vk::TRUE {
				log::warn!("Device {} does not support bufferDeviceAddress ext", name);
				continue;
			}
			if !DeviceContext::has_minimum_queue_families_reqs(
				instance,
				device,
				surface_loader,
				tmp_surface,
			) {
				log::warn!("Device {} has no suitable queue families, skipping", name);
				continue;
			}

			score += properties.limits.max_image_dimension2_d;
			candidates.push((score, device, name));
		}
		if candidates.is_empty() {
			panic!("Could not find suitable GPU");
		}
		candidates.sort_by(|a, b| b.0.cmp(&a.0));
		let &(score, device, ref name) = &candidates[0];
		log::info!("picked device {}: score = {}", name, score);
		device
	}

	fn create_logical_device(
		instance: &Instance,
		phys_device: vk::PhysicalDevice,
		graphics_idx: u32,
	) -> Device {
		// queue
		let prio: f32 = 0.;
		let device_queue_create_info = vk::DeviceQueueCreateInfo {
			queue_family_index: graphics_idx,
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
		let mut vk11_features = vk::PhysicalDeviceVulkan11Features {
			shader_draw_parameters: vk::TRUE,
			..Default::default()
		};
		let mut vk12_features = vk::PhysicalDeviceVulkan12Features {
			buffer_device_address: vk::TRUE,
			..Default::default()
		};
		let device_create_info = vk::DeviceCreateInfo {
			p_queue_create_infos: &device_queue_create_info,
			queue_create_info_count: 1,
			enabled_extension_count: DEVICE_REQUIRED_EXTENSIONS.len() as u32,
			pp_enabled_extension_names: device_extensions.as_ptr(),
			..Default::default()
		}
		.push_next(&mut dynamic_rendering)
		.push_next(&mut extended_dynamic_state)
		.push_next(&mut vk11_features)
		.push_next(&mut vk12_features);

		let device = unsafe {
			instance
				.create_device(phys_device, &device_create_info, None)
				.expect("Should have been able to create logical device")
		};
		device
	}

	pub fn has_minimum_queue_families_reqs(
		instance: &Instance,
		device: vk::PhysicalDevice,
		surface_loader: &surface::Instance,
		tmp_surface: &vk::SurfaceKHR,
	) -> bool {
		let queue_family_properties =
			unsafe { instance.get_physical_device_queue_family_properties(device) };
		let supports_graphics = queue_family_properties
			.iter()
			.any(|properties| properties.queue_flags.contains(vk::QueueFlags::GRAPHICS));
		let supports_present = queue_family_properties
			.iter()
			.enumerate()
			.any(|(idx, _)| unsafe {
				surface_loader
					.get_physical_device_surface_support(device, idx as u32, *tmp_surface)
					.expect("Should be able to check for present support")
			});

		supports_graphics && supports_present
	}

	fn find_queue_families(
		instance_ctx: &InstanceContext,
		phys_device: &vk::PhysicalDevice,
		tmp_surface: &vk::SurfaceKHR,
	) -> (u32, u32) {
		let instance = instance_ctx.instance();
		let surface_loader = instance_ctx.surface_loader();

		let queue_family_properties =
			unsafe { instance.get_physical_device_queue_family_properties(*phys_device) };

		let supports_present = |index: usize| -> bool {
			unsafe {
				surface_loader
					.get_physical_device_surface_support(*phys_device, index as u32, *tmp_surface)
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
		log::info!(
			"graphics index: [{}], present index: [{}]",
			graphics_idx,
			present_idx
		);
		(graphics_idx as u32, present_idx as u32)
	}
}
