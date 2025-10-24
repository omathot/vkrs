use super::{DeviceContext, InstanceContext, Window};

pub struct VkCore {
	pub instance_ctx: InstanceContext,
	pub device_ctx: DeviceContext,
}
impl VkCore {
	pub fn new(window: &Window) -> VkCore {
		let instance_ctx = InstanceContext::new();
		let device_ctx = DeviceContext::new(&instance_ctx, window);

		VkCore {
			instance_ctx,
			device_ctx,
		}
	}
	pub fn cleanup(&self) {
		// debug messenger
		unsafe {
			self.instance_ctx
				.debug_utils_loader
				.destroy_debug_utils_messenger(self.instance_ctx.debug_messenger, None);
		}
		// device
		unsafe {
			self.device_ctx.device().destroy_device(None);
		}
		// instance
		unsafe {
			self.instance_ctx.instance().destroy_instance(None);
		}
	}
}
