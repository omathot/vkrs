use ash::{Entry, Instance, vk};
use common::*;
use std::ffi::{CStr, CString};
use std::time::Instant;
use std::{error::Error, result::Result};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;
mod common;
mod utils;
mod window;

struct Application {
	instance: Option<Instance>,
	window: Option<Window>,

	last_frame: Instant,
}

impl Application {
	pub fn new() -> Application {
		log::info!("Building application!");
		Application {
			instance: None,
			window: None,
			last_frame: Instant::now(),
		}
	}
	fn update(&self, dt: f32) {}
	fn render(&self) {}
	fn cleanup(&mut self) {}

	fn create_instance() -> Result<Instance, vk::Result> {
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

		// make required layers
		let required_layers = Application::get_required_layers(&entry);

		// make required extensions
		let required_extensions: CStringArray = Application::get_required_extensions();

		let app_info = vk::ApplicationInfo {
			application_version: vk::make_api_version(0, 1, 0, 0),
			api_version: vk::make_api_version(0, 1, 4, 0),
			..Default::default()
		};
		let create_info = vk::InstanceCreateInfo {
			p_application_info: &app_info,
			enabled_layer_count: required_layers.len() as u32,
			pp_enabled_layer_names: required_layers.as_ptr(),
			enabled_extension_count: required_extensions.len() as u32,
			pp_enabled_extension_names: required_extensions.as_ptr(),
			..Default::default()
		};

		let instance = unsafe { entry.create_instance(&create_info, None)? };
		Ok(instance)
	}

	pub fn init_vulkan(&mut self) {
		self.instance =
			Some(Application::create_instance().expect("Should have been able to create instance"));
	}

	fn get_required_extensions() -> CStringArray {
		let mut extension_names = WL_REQUIRED_EXTENSIONS.to_vec();
		if ENABLE_VALIDATION_LAYERS {
			let dbg_utils_name = vk::EXT_DEBUG_UTILS_NAME
				.to_str()
				.expect("Debug utils extension name should be valid");
			extension_names.push(dbg_utils_name);
		}
		log::info!("{} extensions:", extension_names.len());
		extension_names
			.iter()
			.for_each(|extension| log::info!("\t{}", extension));
		CStringArray::from(extension_names)
	}

	fn get_required_layers(entry: &Entry) -> CStringArray {
		// query layers
		let layer_properties = unsafe {
			entry
				.enumerate_instance_layer_properties()
				.expect("Should have been able to get layer properties from entry")
		};
		log::info!(
			"{} available instance layer properties:",
			layer_properties.len()
		);
		layer_properties
			.iter()
			.for_each(|property| log::info!("\t{:?}", property.layer_name_as_c_str().unwrap()));
		println!("");

		// make required layers
		let mut required_layers: Vec<&str> = Vec::new();
		if ENABLE_VALIDATION_LAYERS {
			required_layers.extend(VALIDATION_LAYERS.iter());
		}
		if required_layers.iter().any(|required_layer| {
			let cstr_name = CString::new(*required_layer)
				.expect("Should have been able to create CString from required layer name");
			!layer_properties
				.iter()
				.any(|property| property.layer_name_as_c_str().unwrap() == cstr_name.as_c_str())
		}) {
			log::error!("One or more required layers are not supported!");
		}
		// needed conversion to put as *const *const i8 in create_info
		let required_layer_names: Vec<CString> = required_layers
			.iter()
			.map(|layer| CString::new(*layer).unwrap())
			.collect();
		let required_layer_names_ptrs: Vec<*const i8> = required_layer_names
			.iter()
			.map(|layer| layer.as_ptr())
			.collect();
		CStringArray::new(required_layer_names, required_layer_names_ptrs)
	}
}

fn convert_vec_str_to_i8(v: &Vec<&str>) -> (Vec<CString>, Vec<*const i8>) {
	let v_cstr: Vec<CString> = v.iter().map(|item| CString::new(*item).unwrap()).collect();
	let v_ptr: Vec<*const i8> = v_cstr.iter().map(|item| item.as_ptr()).collect();
	(v_cstr, v_ptr)
}
fn main() {
	env_logger::builder()
		.filter_module("learnvulkan", log::LevelFilter::Info)
		.init();

	let event_loop = EventLoop::new().expect("Should have been able to get event loop");
	event_loop.set_control_flow(ControlFlow::Poll);

	let mut app = Application::new();
	event_loop
		.run_app(&mut app)
		.expect("Should have been able to run app loop");
}
