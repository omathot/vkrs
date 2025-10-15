use ash::ext::debug_utils;

use super::{Application, ENABLE_VALIDATION_LAYERS, vk};
use std::ffi::{CStr, c_void};

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

	pub fn transition_img_layout(
		&self,
		img_idx: u32,
		old_layout: vk::ImageLayout,
		new_layout: vk::ImageLayout,
		src_access_mask: vk::AccessFlags2,
		dst_access_mask: vk::AccessFlags2,
		src_stage_mask: vk::PipelineStageFlags2,
		dst_stage_mask: vk::PipelineStageFlags2,
	) {
		let barrier = vk::ImageMemoryBarrier2 {
			src_stage_mask: src_stage_mask,
			src_access_mask: src_access_mask,
			dst_stage_mask: dst_stage_mask,
			dst_access_mask: dst_access_mask,
			old_layout: old_layout,
			new_layout: new_layout,
			src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
			dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
			image: *self
				.swapchain_imgs
				.as_ref()
				.unwrap()
				.get(img_idx as usize)
				.expect("img_idx should always be valid for swapchain_imgs"),
			subresource_range: vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			},
			..Default::default()
		};
		let deps_info = vk::DependencyInfo {
			image_memory_barrier_count: 1,
			p_image_memory_barriers: &barrier,
			..Default::default()
		};
		unsafe {
			self.device
				.as_ref()
				.unwrap()
				.cmd_pipeline_barrier2(self.command_buff.unwrap(), &deps_info);
		}
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
