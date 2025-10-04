use super::{Application, vk};
use crate::common::*;
use std::{
	collections::HashMap,
	ffi::{CStr, CString, c_char},
	task::Waker,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
	#[error("Init error: `{0}`")]
	Init(String),
	#[error("Unknown error")]
	Unknown,
}

impl Application {
	pub fn update(&self, dt: f32) {}
	pub fn render(&self) {}
	pub fn cleanup(&mut self) {
		if let (Some(loader), Some(messenger)) = (&self.debug_utils_loader, self.debug_messenger) {
			unsafe { loader.destroy_debug_utils_messenger(messenger, None) };
		}
		if let Some(instance) = &self.instance {
			unsafe { instance.destroy_instance(None) };
		}
	}

	fn create_instance(&mut self) {
		// make required layers
		let required_layers = self
			.get_required_layers()
			.expect("Should have gotten required layers");

		// make required extensions
		let required_extensions: Vec<*const c_char> = Application::get_required_extensions();

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

	pub fn init_vulkan(&mut self) {
		self.create_instance();
		self.setup_debug_messenger();
		self.pick_physical_device();
		self.create_logical_device();
	}

	fn get_required_extensions() -> Vec<*const c_char> {
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

	fn get_required_layers(&self) -> Result<CStringArray, AppError> {
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
			log::error!("One or more required layers are not supported!");
			return Err(AppError::Init(
				"One or more required layers are not supported!".to_string(),
			));
		}
		let required_layer_names: Vec<CString> = required_layers
			.iter()
			.map(|layer| CString::new(*layer).unwrap())
			.collect();
		let required_layer_names_ptrs: Vec<*const c_char> = required_layer_names
			.iter()
			.map(|layer| layer.as_ptr())
			.collect();
		Ok(CStringArray::new(
			required_layer_names,
			required_layer_names_ptrs,
		))
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
			// requirements
			if features.geometry_shader != vk::TRUE {
				log::warn!("device {} does not have a geometry shader", name);
				continue;
			}
			if self.find_queue_families(device).is_none() {
				log::warn!("Device {} has no suitable queue families, skipping", name);
				continue;
			}

			let mut score = 0;
			log::info!("Checking device {:?}", name);
			if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
				score += 1000;
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
		self.graphics_index = self.find_queue_families(device).unwrap();
	}

	fn create_logical_device(&mut self) {
		let instance = self.instance.as_ref().unwrap();
		let prio: f32 = 0.;
		//features
		let mut dynamic_rendering = vk::PhysicalDeviceDynamicRenderingFeatures {
			dynamic_rendering: vk::TRUE,
			..Default::default()
		};
		let mut extended_dynamic_state = vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT {
			extended_dynamic_state: vk::TRUE,
			..Default::default()
		};
		let device_queue_create_info = vk::DeviceQueueCreateInfo {
			queue_family_index: self.graphics_index,
			p_queue_priorities: &prio,
			queue_count: 1,
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
					.create_device(self.physical_device.unwrap(), &device_create_info, None)
					.expect("Should have been able to create logical device"),
			)
		};
		self.graphics_queue = unsafe {
			Some(
				self.device
					.as_ref()
					.unwrap()
					.get_device_queue(self.graphics_index, 0),
			)
		};
	}

	fn find_queue_families(&self, device: vk::PhysicalDevice) -> Option<u32> {
		let instance = self.instance.as_ref().unwrap();
		let queue_family_properties =
			unsafe { instance.get_physical_device_queue_family_properties(device) };
		queue_family_properties
			.iter()
			.position(|queue| queue.queue_flags.contains(vk::QueueFlags::GRAPHICS))
			.map(|index| index as u32)
	}
}
