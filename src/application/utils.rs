use ash::ext::debug_utils;

use super::{Application, ENABLE_VALIDATION_LAYERS, vk};
use std::ffi::CStr;

impl Application {
	pub fn setup_debug_messenger(&mut self) {
		if !ENABLE_VALIDATION_LAYERS {
			return;
		}
		let instance = self
			.instance
			.as_ref()
			.expect("Instance should be init in new before debug messenger");
		let debug_utils_loader = debug_utils::Instance::new(&self.entry, instance);
		let severity_flags = vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
			| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
			| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
		let message_type_flags = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
			| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
			| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;
		let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
			message_severity: severity_flags,
			message_type: message_type_flags,
			pfn_user_callback: Some(Application::debug_callback),
			..Default::default()
		};
		let messenger = unsafe {
			debug_utils_loader
				.create_debug_utils_messenger(&messenger_create_info, None)
				.expect("Should have been able to create debug messenger")
		};
		self.debug_utils_loader = Some(debug_utils_loader);
		self.debug_messenger = Some(messenger);
	}

	pub unsafe extern "system" fn debug_callback(
		msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
		msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
		p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
		_p_user_data: *mut std::ffi::c_void,
	) -> vk::Bool32 {
		let callback_data = unsafe { *p_callback_data };
		let msg = unsafe { CStr::from_ptr(callback_data.p_message) }
			.to_str()
			.expect("Should have been able to parse message");
		match msg_severity {
			vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
				println!("-- Validation layer [{:?}]: {}", msg_type, msg);
			}
			vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
				println!("-- Validation layer [{:?}]: {}", msg_type, msg);
			}
			vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
				println!("-- Validation layer [{:?}]: {}", msg_type, msg);
			}
			_ => {}
		}
		vk::FALSE
	}
}
