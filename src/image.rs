use nalgebra::Vector3;

pub struct Image {
    width: u32,
    pixels: Vec<Color>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            pixels: vec![Color::BLACK; (width * height) as usize],
        }
    }

    pub fn pixels(&self) -> &[Color] {
        self.pixels.as_slice()
    }

    pub fn pixels_mut(&mut self) -> &mut [Color] {
        self.pixels.as_mut_slice()
    }

    pub fn pixel(&mut self, x: u32, y: u32) -> &mut Color {
        &mut self.pixels[(y * self.width + x) as usize]
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.pixels.len() as u32 / self.width
    }

    // Build the image as the plaintext-based PPM format.
    pub fn ppm(&self) -> String {
        let mut ppm = "P3\n".to_owned();
        ppm += &self.width().to_string();
        ppm += " ";
        ppm += &self.height().to_string();

        ppm += "\n255\n";

        // write pixels

        let mut row_counter = 0;
        for color in self.pixels() {
            ppm += &color.r.to_string();
            ppm += " ";
            ppm += &color.g.to_string();
            ppm += " ";
            ppm += &color.b.to_string();
            ppm += " ";

            row_counter += 1;

            if row_counter == self.width() {
                ppm += "\n";
                row_counter = 0;
            }
        }

        ppm
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    pub fn from_float_vector(rgb: &Vector3<f64>) -> Self {
        Self {
            r: (rgb[0] * 255.999) as u8,
            g: (rgb[1] * 255.999) as u8,
            b: (rgb[2] * 255.999) as u8,
        }
    }
}
