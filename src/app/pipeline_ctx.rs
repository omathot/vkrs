use super::vk;

pub struct PipelineContext {
	pub pipeline_layout: vk::PipelineLayout,
	pub graphics_pipeline: vk::Pipeline,

	pub command_pool: vk::CommandPool, // manages the memory used to store buffers
	pub command_buff: vk::CommandBuffer,
}
