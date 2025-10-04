use super::{Application, vk};
use crate::common::*;
use std::ffi::{CString, c_char};
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
		let required_extensions: CStringArray = Application::get_required_extensions();

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
	}

	fn get_required_extensions() -> CStringArray {
		let mut extension_names = WL_REQUIRED_EXTENSIONS.to_vec();
		if ENABLE_VALIDATION_LAYERS {
			let dbg_utils_name = vk::EXT_DEBUG_UTILS_NAME
				.to_str()
				.expect("Debug utils extension name should be valid");
			extension_names.push(dbg_utils_name);
		}
		log::info!("{} extensions:", extension_names.len());
		extension_names
			.iter()
			.for_each(|extension| log::info!("\t{}", extension));
		CStringArray::from(extension_names)
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

	fn pick_physical_device(&self) {
		let instance = self
			.instance
			.as_ref()
			.expect("Instance should be init before physical device");
		let devices = unsafe {
			instance
				.enumerate_physical_devices()
				.expect("Should be able to list physical devices")
		};
		if devices.is_empty() {}
	}
}
