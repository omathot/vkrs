use super::{Application, ENABLE_VALIDATION_LAYERS, vk};
use std::ffi::CStr;

impl Application {
	pub fn setup_debug_messenger(&mut self) {
		if !ENABLE_VALIDATION_LAYERS {
			return;
		}
		let severity_flags = vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
			| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
			| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
		let message_type_flags = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
			| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
			| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;
		let messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
			message_severity: severity_flags,
			message_type: message_type_flags,
			// pfn_user_callback: &Application::debug_callback,
			..Default::default()
		};
		// if let Some(instance) = &self.instance {
		// 	instance.create_debug_utils_messenger();
		// }
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
				log::error!("{:?}, {}", msg_type, msg);
			}
			vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
				log::warn!("{:?}, {}", msg_type, msg);
			}
			vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
				log::info!("{:?}, {}", msg_type, msg);
			}
			_ => {}
		}
		vk::FALSE
	}
}
