use ash::ext::debug_utils;

use super::{Application, ENABLE_VALIDATION_LAYERS, vk};
use std::ffi::{CStr, c_void};

impl Application {
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
		// unsafe {
		// 	self.device
		// 		.as_ref()
		// 		.unwrap()
		// 		.cmd_pipeline_barrier2(self.command_buff.unwrap(), &deps_info);
		// }
	}
}
