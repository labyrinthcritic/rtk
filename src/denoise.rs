use image::{ImageBuffer, Rgb};

pub fn denoise(_image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<(), ()> {
    // temporary stuff to test that ffi is working
    let mut device = oidn::Device::new().unwrap();
    let mut buffer = device.create_buffer(1000).unwrap();

    let slice = buffer.as_mut_slice();

    for item in slice.iter_mut() {
        *item = 0.0;
    }
    Ok(())
}
