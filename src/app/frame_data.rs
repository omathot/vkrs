use super::{Device, vk};

pub struct FrameData {
	pub cmd_buff: vk::CommandBuffer, // 1 cmd buff per frame, allocated from cmd_pool in vkSwap
	pub img_available: vk::Semaphore,
	pub render_finished: vk::Semaphore,
	pub draw_fence: vk::Fence,
}

impl FrameData {
	pub fn new(device: &Device, command_pool: vk::CommandPool) -> FrameData {
		let cmd_buff = FrameData::create_command_buff(device, command_pool);
		let img_available = unsafe {
			device
				.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
				.expect("Should have been able to create present_finished semaphore")
		};
		let render_finished = unsafe {
			device
				.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
				.expect("Should have been able to create render_ready semaphore")
		};
		let draw_fence = unsafe {
			device
				.create_fence(
					&vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
					None,
				)
				.expect("Should have been able to create draw_fence")
		};

		FrameData {
			cmd_buff,
			img_available,
			render_finished,
			draw_fence,
		}
	}

	fn create_command_buff(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
		let alloc_info = vk::CommandBufferAllocateInfo {
			command_pool: command_pool,
			level: vk::CommandBufferLevel::PRIMARY,
			command_buffer_count: 1,
			..Default::default()
		};
		let command_buff = unsafe {
			device
				.allocate_command_buffers(&alloc_info)
				.expect("Should have been able to allocate command buff")[0] // only creating one for now
		};

		command_buff
	}
}
