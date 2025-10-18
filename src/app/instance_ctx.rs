use super::{CStringArray, Entry, Instance, common::*, debug_utils, surface, vk};
use std::ffi::{CStr, CString, c_char, c_void};

pub struct InstanceContext {
	pub entry: Entry,
	pub instance: Instance,
	pub surface_loader: surface::Instance,
	#[cfg(feature = "validation")]
	pub debug_utils_loader: debug_utils::Instance,
	#[cfg(feature = "validation")]
	pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl InstanceContext {
	pub fn new() -> InstanceContext {
		let entry = Entry::linked();
		let required_layers = InstanceContext::get_required_layers(&entry);
		let required_extensions: Vec<*const c_char> = InstanceContext::get_required_extensions();
		#[cfg(debug_assertions)]
		{
			// query all extensions
			let available_extensions = unsafe {
				entry
					.enumerate_instance_extension_properties(None)
					.expect("Should have been able to get instance extension properties")
			};
			log::info!("{} available extensions:", available_extensions.len());
			available_extensions.iter().for_each(|extension| {
				log::info!("\t{:?}", extension.extension_name_as_c_str().unwrap())
			});
			println!("");
		}

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
			#[cfg(target_os = "macos")]
			flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
			..Default::default()
		};

		let instance = unsafe {
			entry
				.create_instance(&create_info, None)
				.expect("Should have been able to create instance")
		};
		#[cfg(feature = "validation")]
		let (debug_loader, debug_messenger) = InstanceContext::setup_debug_messenger(&instance, &entry);
		let surface_loader = surface::Instance::new(&entry, &instance);
		InstanceContext {
			entry: entry,
			instance: instance,
			surface_loader: surface_loader,
			#[cfg(feature = "validation")]
			debug_utils_loader: debug_loader,
			#[cfg(feature = "validation")]
			debug_messenger: debug_messenger,
		}
	}
	pub fn entry(&self) -> &Entry {
		&self.entry
	}
	pub fn instance(&self) -> &Instance {
		&self.instance
	}
	pub fn surface_loader(&self) -> &surface::Instance {
		&self.surface_loader
	}

	fn setup_debug_messenger(
		instance: &Instance,
		entry: &Entry,
	) -> (debug_utils::Instance, vk::DebugUtilsMessengerEXT) {
		let debug_utils_loader = debug_utils::Instance::new(entry, instance);
		let severity_flags = vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
			| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
			| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
		let message_type_flags = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
			| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
			| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;
		let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
			message_severity: severity_flags,
			message_type: message_type_flags,
			pfn_user_callback: Some(InstanceContext::debug_callback),
			..Default::default()
		};
		let messenger = unsafe {
			debug_utils_loader
				.create_debug_utils_messenger(&messenger_create_info, None)
				.expect("Should have been able to create debug messenger")
		};
		(debug_utils_loader, messenger)
	}

	fn get_required_extensions() -> Vec<*const c_char> {
		let mut extension_names = REQUIRED_INSTANCE_EXTENSIONS.to_vec();
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

	fn get_required_layers(entry: &Entry) -> CStringArray {
		// query layers
		let layer_properties = unsafe {
			entry
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

	pub unsafe extern "system" fn debug_callback(
		msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
		msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
		p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
		_p_user_data: *mut c_void,
	) -> vk::Bool32 {
		let callback_data = unsafe { *p_callback_data };
		let msg = unsafe { CStr::from_ptr(callback_data.p_message) }
			.to_str()
			.expect("Should have been able to parse message");
		match msg_severity {
			vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
				log::error!("-- Validation layer -- [{:?}]: {}", msg_type, msg);
			}
			vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
				log::warn!("-- Validation layer -- [{:?}]: {}", msg_type, msg);
			}
			vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
				log::info!("-- Validation layer -- [{:?}], {}", msg_type, msg);
			}
			_ => {}
		}
		vk::FALSE
	}
}
