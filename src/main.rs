use winit::event_loop::{ControlFlow, EventLoop};

mod application;
use application::*;

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
