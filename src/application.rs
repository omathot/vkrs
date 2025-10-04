use ash::{Entry, Instance, vk};
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
	pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

	pub last_frame: Instant,
}
