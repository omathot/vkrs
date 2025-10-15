use winit::event_loop::{ControlFlow, EventLoop};

mod app;
use app::*;

// event loop owned by main rn. seems sort of necessary since need to pass app into run_app method.
// will think about this more when its more relevant
fn main() {
	let event_loop = EventLoop::new().expect("Should have been able to get event loop");
	event_loop.set_control_flow(ControlFlow::Poll);

	let mut app = Application::new();
	event_loop
		.run_app(&mut app)
		.expect("Should have been able to run app loop");
}
