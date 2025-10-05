use ash::ext::debug_utils;
use ash::khr::surface;
use ash::{Device, Entry, Instance, vk};
use std::time::Instant;
use winit::window::Window;

pub mod common;
pub mod init;
pub mod utils;
pub mod window;
pub use common::*;

pub struct Application {
	pub entry: Entry,
	pub instance: Option<Instance>,
	pub window: Option<Window>,
	pub debug_utils_loader: Option<debug_utils::Instance>,
	pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

	pub physical_device: Option<vk::PhysicalDevice>, // gpu/cpu/other
	pub device: Option<Device>, // logical connection - 'i am running vk on this device'
	pub graphics_index: u32,
	pub present_index: u32,
	pub graphics_queue: Option<vk::Queue>,

	pub surface_loader: Option<surface::Instance>,
	pub surface: Option<vk::SurfaceKHR>,

	pub last_frame: Instant,
}

impl Application {
	pub fn new() -> Application {
		log::info!("Building application!");
		let entry = Entry::linked();
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
		Application {
			entry,
			instance: None,
			window: None,
			debug_utils_loader: None,
			debug_messenger: None,
			physical_device: None,
			device: None,
			graphics_index: 0,
			present_index: 0,
			graphics_queue: None,
			surface_loader: None,
			surface: None,
			last_frame: Instant::now(),
		}
	}
}
