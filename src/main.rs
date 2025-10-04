use ash::{Entry, Instance, vk};
use std::ffi::CStr;
use std::time::Instant;
use std::{error::Error, result::Result};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};
mod utils;

#[cfg(debug_assertions)]
static ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
static ENABLE_VALIDATION_LAYERS: bool = false;

struct Application {
	instance: Option<Instance>,
	window: Option<Window>,

	last_frame: Instant,
}
impl ApplicationHandler for Application {
	// Init our graphics context on resumed because of certain platforms
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.instance.is_none() {
			init_vulkan(self);
		}
		if self.window.is_none() {
			let attributes = WindowAttributes::default().with_title("Vulkan rs");
			self.window = Some(
				event_loop
					.create_window(attributes)
					.expect("Should have been able to create window from event loop"),
			);
		}
	}
	fn suspended(&mut self, event_loop: &ActiveEventLoop) {}
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested => {
				self.cleanup();
				event_loop.exit();
			}
			WindowEvent::Resized(size) => { /* resize */ }
			WindowEvent::RedrawRequested => {
				// tick
				let now = Instant::now();
				let dt = now.duration_since(self.last_frame).as_secs_f32();
				self.last_frame = now;

				// game logic
				self.update(dt);
				// render
				self.render();
				if let Some(window) = &self.window {
					window.request_redraw();
				}
			}
			_ => {}
		}
	}
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
}

fn create_instance() -> Result<Instance, vk::Result> {
	let entry = Entry::linked();
	let extensions = unsafe {
		entry
			.enumerate_instance_extension_properties(None)
			.expect("Should have been able to get instance extension properties")
	};
	log::info!("{} vailable extensions", extensions.len());
	for extension in extensions {
		unsafe {
			log::info!(
				"\t{}",
				CStr::from_ptr(extension.extension_name.as_ptr())
					.to_str()
					.expect("Should have been able to get CStr from vk extension property")
			);
		}
	}
	let app_info = vk::ApplicationInfo {
		application_version: vk::make_api_version(0, 1, 0, 0),
		api_version: vk::make_api_version(0, 1, 4, 0),
		..Default::default()
	};
	let create_info = vk::InstanceCreateInfo {
		p_application_info: &app_info,
		..Default::default()
	};

	// query extensions
	let instance = unsafe { entry.create_instance(&create_info, None)? };
	Ok(instance)
}

fn init_vulkan(app: &mut Application) {
	app.instance = Some(create_instance().expect("Should have been able to create instance"));
}
fn main() {
	env_logger::builder()
		.filter_module("learnvulkan", log::LevelFilter::Info)
		.init();

	let event_loop = EventLoop::new().expect("Should have been able to get event loop");
	event_loop.set_control_flow(ControlFlow::Poll);

	let mut app = Application::new().expect("Should have been able to create application");
	event_loop
		.run_app(&mut app)
		.expect("Should have been able to run app loop");
}
