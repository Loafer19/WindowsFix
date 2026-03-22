use windows::Win32::System::Services::{CloseServiceHandle, SC_HANDLE};

/// Convert a Rust `&str` to a null-terminated UTF-16 `Vec<u16>` for Win32 API calls.
pub fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// RAII guard for `SC_HANDLE` that automatically closes the handle when dropped.
pub struct ScHandle(pub SC_HANDLE);

impl ScHandle {
    pub fn new(handle: SC_HANDLE) -> Self {
        Self(handle)
    }

    pub fn get(&self) -> SC_HANDLE {
        self.0
    }
}

impl Drop for ScHandle {
    fn drop(&mut self) {
        unsafe {
            CloseServiceHandle(self.0).ok();
        }
    }
}
