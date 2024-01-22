use std::ffi::{c_char, CStr, CString};

pub use oidn_sys as sys;

/// Any error reported by the device.
#[derive(Clone)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Device {
    device: sys::OIDNDevice,
}

impl Device {
    pub fn new() -> Result<Self> {
        unsafe {
            let device = sys::oidnNewDevice(sys::OIDNDeviceType_OIDN_DEVICE_TYPE_DEFAULT);
            sys::oidnCommitDevice(device);

            let device = Self { device };
            device.get_error()?;

            Ok(device)
        }
    }

    pub fn get_error(&self) -> Result<()> {
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

    pub fn create_buffer(&self, len: usize) -> Result<Buffer> {
        Buffer::new(self, len)
    }

    pub fn create_filter(&self) -> Result<Filter> {
        Filter::new(self)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            sys::oidnReleaseDevice(self.device);
        }
    }
}

pub struct Buffer<'a> {
    _device: &'a Device,
    buffer: sys::OIDNBuffer,
    len: usize,
}

impl<'a> Buffer<'a> {
    fn new(device: &'a Device, len: usize) -> Result<Self> {
        unsafe {
            let buffer = sys::oidnNewBuffer(device.device, len * std::mem::size_of::<f32>());
            device.get_error()?;
            Ok(Buffer {
                _device: device,
                buffer,
                len,
            })
        }
    }

    pub fn as_slice(&mut self) -> &'a [f32] {
        unsafe {
            let buffer_data = sys::oidnGetBufferData(self.buffer);
            assert!(!buffer_data.is_null());

            std::slice::from_raw_parts(buffer_data.cast(), self.len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &'a mut [f32] {
        unsafe {
            let buffer_data = sys::oidnGetBufferData(self.buffer);
            assert!(!buffer_data.is_null());

            std::slice::from_raw_parts_mut(buffer_data.cast(), self.len)
        }
    }
}

impl Drop for Buffer<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::oidnReleaseBuffer(self.buffer);
        }
    }
}

pub struct Filter<'a> {
    device: &'a Device,
    filter: sys::OIDNFilter,
    /// This field exists to restrict the lifetime of the filter.
    color_image: Option<&'a Buffer<'a>>,
}

impl<'a> Filter<'a> {
    fn new(device: &'a Device) -> Result<Self> {
        unsafe {
            let rt = CString::new("RT").unwrap();
            let filter = sys::oidnNewFilter(device.device, rt.as_ptr());
            device.get_error()?;
            Ok(Filter {
                device,
                filter,
                color_image: None,
            })
        }
    }

    fn set_image(
        &mut self,
        buffer: &'a Buffer,
        width: usize,
        height: usize,
        name: &CStr,
    ) -> Result<()> {
        unsafe {
            sys::oidnSetFilterImage(
                self.filter,
                name.as_ptr(),
                buffer.buffer,
                sys::OIDNFormat_OIDN_FORMAT_FLOAT3,
                width,
                height,
                0,
                0,
                0,
            );
        }

        self.device.get_error()?;

        self.color_image = Some(buffer);

        Ok(())
    }

    pub fn set_color_image(
        &mut self,
        buffer: &'a Buffer,
        width: usize,
        height: usize,
    ) -> Result<()> {
        let color = CString::new("color").unwrap();
        self.set_image(buffer, width, height, &color)
    }

    pub fn set_output_image(
        &mut self,
        buffer: &'a Buffer,
        width: usize,
        height: usize,
    ) -> Result<()> {
        let color = CString::new("output").unwrap();
        self.set_image(buffer, width, height, &color)
    }

    pub fn execute(&self) -> Result<()> {
        unsafe {
            sys::oidnCommitFilter(self.filter);
            self.device.get_error()?;
            sys::oidnExecuteFilter(self.filter);
            self.device.get_error()?;
        }

        Ok(())
    }
}

impl Drop for Filter<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::oidnReleaseFilter(self.filter);
        }
    }
}
