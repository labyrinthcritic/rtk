use image::{ImageBuffer, Rgb};

pub fn denoise(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let device = oidn::Device::new().unwrap();
    let mut buffer = device.create_buffer(image.pixels().len() * 3).unwrap();

    let pfm_image = create_pfm(image);

    buffer.as_mut_slice().copy_from_slice(&pfm_image);

    {
        let mut filter = device.create_filter().unwrap();
        filter
            .set_color_image(&buffer, image.width() as usize, image.height() as usize)
            .unwrap();
        filter
            .set_output_image(&buffer, image.width() as usize, image.height() as usize)
            .unwrap();

        filter.execute().unwrap();
    }

    let mut result_pfm = vec![0.0; image.pixels().len() * 3];
    result_pfm.as_mut_slice().copy_from_slice(buffer.as_slice());

    create_image(
        result_pfm.as_slice(),
        image.width() as usize,
        image.height() as usize,
    )
}

fn create_pfm(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<f32> {
    let mut result = Vec::new();

    for pixel in image.pixels() {
        result.push(pixel[0] as f32 / 255.999);
        result.push(pixel[1] as f32 / 255.999);
        result.push(pixel[2] as f32 / 255.999);
    }

    result
}

fn create_image(pfm: &[f32], width: usize, height: usize) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut image = ImageBuffer::new(width as u32, height as u32);

    for (i, chunk) in pfm.chunks(3).enumerate() {
        let x = i % width;
        let y = i / width;

        image.put_pixel(
            x as u32,
            y as u32,
            Rgb([
                (chunk[0] * 255.999) as u8,
                (chunk[1] * 255.999) as u8,
                (chunk[2] * 255.999) as u8,
            ]),
        )
    }

    image
}
