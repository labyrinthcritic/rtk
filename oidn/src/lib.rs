use std::ffi::{c_char, CStr};

pub use oidn_sys as sys;

#[derive(Clone, Debug)]
pub struct Error {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Device {
    device: sys::OIDNDevice,
}

impl Device {
    pub fn new() -> Result<Self> {
        unsafe {
            let device = sys::oidnNewDevice(sys::OIDNDeviceType_OIDN_DEVICE_TYPE_DEFAULT);
            sys::oidnCommitDevice(device);

            let mut device = Self { device };
            device.get_error()?;

            Ok(device)
        }
    }

    pub fn get_error(&mut self) -> Result<()> {
        let mut c_string_ptr: *const c_char = std::ptr::null();
        unsafe {
            if sys::oidnGetDeviceError(self.device, &mut c_string_ptr)
                != sys::OIDNError_OIDN_ERROR_NONE
            {
                return Err(Error {
                    message: CStr::from_ptr(c_string_ptr).to_string_lossy().to_string(),
                });
            }
        }

        Ok(())
    }

    pub fn create_buffer(&mut self, len: usize) -> Result<Buffer> {
        Buffer::new(self, len)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            sys::oidnReleaseDevice(self.device);
        }
    }
}

pub struct Buffer {
    buffer: sys::OIDNBuffer,
    len: usize,
}

impl Buffer {
    fn new(device: &mut Device, len: usize) -> Result<Self> {
        unsafe {
            let buffer = sys::oidnNewBuffer(device.device, len * std::mem::size_of::<f32>());
            device.get_error()?;
            Ok(Buffer { buffer, len })
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        unsafe {
            let buffer_data = sys::oidnGetBufferData(self.buffer);
            std::slice::from_raw_parts_mut(buffer_data.cast(), self.len)
        }
    }
}
