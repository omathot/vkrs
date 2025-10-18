use super::{Application, vk};
impl Application {
	pub fn create_shader_module(&self, code: &'static [u8]) -> vk::ShaderModule {
		let create_info = vk::ShaderModuleCreateInfo {
			code_size: code.len(),
			p_code: code.as_ptr() as *const u32,
			..Default::default()
		};
		unsafe {
			self.device
				.as_ref()
				.unwrap()
				.create_shader_module(&create_info, None)
				.expect("Should have been able to create shader module")
		}
	}
}
