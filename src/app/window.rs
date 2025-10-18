use super::{Application, Instant, VkCore, VkSwap};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

impl ApplicationHandler for Application {
	// Init our graphics context on resumed because of certain platforms (e.g. Android)
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_none() {
			let attributes = WindowAttributes::default().with_title("Vulkan rs");
			self.window = Some(
				event_loop
					.create_window(attributes)
					.expect("Should have been able to create window from event loop"),
			);
		}
		// Permanent VK
		if self.vk.is_none() {
			VkCore::new(self.window.as_ref().unwrap());
		}
		// recreates on each resumed signal
		if self.vk_swapchain.is_none() {
			VkSwap::new();
		}
		// if self.instance.is_none() {
		// 	Application::init_vulkan(self);
		// }
	}
	fn suspended(&mut self, event_loop: &ActiveEventLoop) {
		// TODO: Cleanup non persistent vk objects (surface, imgs, swapchain)
	}
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
				self.draw_frame();
				if let Some(window) = &self.window {
					window.request_redraw();
				}
			}
			_ => {}
		}
	}
}
