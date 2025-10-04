use std::ffi::{CString, c_char};

#[cfg(debug_assertions)]
pub static ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub static ENABLE_VALIDATION_LAYERS: bool = false;

pub static VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
// wayland required extensions. will find better solution to this later
pub static WL_REQUIRED_EXTENSIONS: [&str; 2] = ["VK_KHR_surface", "VK_KHR_wayland_surface"];

// the only purpose of this struct is to keep the CString alive as long as the *const c_char
// otherwise we have to juggle both all the time to keep chars valid
pub struct CStringArray {
	strings: Vec<CString>,
	ptrs: Vec<*const c_char>,
}

impl From<&[&str]> for CStringArray {
	fn from(value: &[&str]) -> Self {
		let strings: Vec<CString> = value.iter().map(|s| CString::new(*s).unwrap()).collect();
		let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
		CStringArray { strings, ptrs }
	}
}

impl From<Vec<&str>> for CStringArray {
	fn from(value: Vec<&str>) -> Self {
		let strings: Vec<CString> = value.iter().map(|s| CString::new(*s).unwrap()).collect();
		let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
		CStringArray { strings, ptrs }
	}
}

impl CStringArray {
	pub fn new(strings: Vec<CString>, ptrs: Vec<*const c_char>) -> CStringArray {
		CStringArray { strings, ptrs }
	}
	pub fn as_ptr(&self) -> *const *const c_char {
		self.ptrs.as_ptr() as *const *const c_char
	}
	pub fn len(&self) -> usize {
		self.ptrs.len()
	}
}
