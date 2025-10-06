use super::{Application, vk};
impl Application {
	pub fn has_minimum_queue_families_reqs(&self, device: vk::PhysicalDevice) -> bool {
		let instance = self.instance.as_ref().unwrap();
		let queue_family_properties =
			unsafe { instance.get_physical_device_queue_family_properties(device) };
		let supports_graphics = queue_family_properties
			.iter()
			.any(|properties| properties.queue_flags.contains(vk::QueueFlags::GRAPHICS));
		let supports_present = queue_family_properties.iter().enumerate().any(|(idx, _)| {
			let loader = self.surface_loader.as_ref().unwrap();
			unsafe {
				loader
					.get_physical_device_surface_support(device, idx as u32, self.surface.unwrap())
					.expect("Should be able to check for present support")
			}
		});

		supports_graphics && supports_present
	}
}
