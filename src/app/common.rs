use ash::vk;
use std::ffi::{CStr, CString, c_char};

// layers
#[cfg(debug_assertions)]
pub static ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub static ENABLE_VALIDATION_LAYERS: bool = false;
pub static VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

// extensions
pub static WL_REQUIRED_EXTENSIONS: [&CStr; 2] =
	[vk::KHR_SURFACE_NAME, vk::KHR_WAYLAND_SURFACE_NAME];

pub static DEVICE_REQUIRED_EXTENSIONS: [&CStr; 3] = [
	vk::KHR_SWAPCHAIN_NAME,
	vk::KHR_SPIRV_1_4_NAME,
	vk::KHR_SYNCHRONIZATION2_NAME,
];

// the only purpose of this struct is to keep the CString alive as long as the *const c_char
// otherwise we have to juggle both to keep chars valid
pub struct CStringArray {
	_strings: Vec<CString>,
	ptrs: Vec<*const c_char>,
}

impl From<&[&str]> for CStringArray {
	fn from(value: &[&str]) -> Self {
		let strings: Vec<CString> = value.iter().map(|s| CString::new(*s).unwrap()).collect();
		let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
		CStringArray {
			_strings: strings,
			ptrs,
		}
	}
}

impl From<Vec<&str>> for CStringArray {
	fn from(value: Vec<&str>) -> Self {
		let strings: Vec<CString> = value.iter().map(|s| CString::new(*s).unwrap()).collect();
		let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
		CStringArray {
			_strings: strings,
			ptrs,
		}
	}
}

impl CStringArray {
	pub fn new(strings: Vec<CString>, ptrs: Vec<*const c_char>) -> CStringArray {
		CStringArray {
			_strings: strings,
			ptrs,
		}
	}
	pub fn as_ptr(&self) -> *const *const c_char {
		self.ptrs.as_ptr() as *const *const c_char
	}
	pub fn len(&self) -> usize {
		self.ptrs.len()
	}
}
