use nalgebra::Vector3;

pub struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    pub fn at(&self, t: f64) -> Vector3<f64> {
        self.origin + t * self.direction
    }
}

pub type Color = Vector3<f64>;

pub struct Renderer {
    aspect_ratio: f64,
    /// In pixels.
    image_width: u32,
    /// In pixels.
    image_height: u32,
    /// In world units.
    viewport_width: f64,
    /// In world units.
    viewport_height: f64,
    /// The distance from the camera center to the viewport center.
    /// This is always orthogonal to the viewport.
    focal_length: f64,
    camera_center: Vector3<f64>,
    /// A vector from the left edge of the viewport to the right edge.
    viewport_u: Vector3<f64>,
    /// A vector from the top edge of the viewport to the bottom edge.
    viewport_v: Vector3<f64>,
    pixel_delta_u: Vector3<f64>,
    pixel_delta_v: Vector3<f64>,
    viewport_upper_left: Vector3<f64>,
    // "pixel00_loc" in the tutorial
    pixel_origin: Vector3<f64>,
}

impl Renderer {
    pub fn new(image_width: u32, image_height: u32) -> Self {
        const VIEWPORT_HEIGHT: f64 = 2.0;

        let aspect_ratio = image_width as f64 / image_height as f64;

        let viewport_width = VIEWPORT_HEIGHT * aspect_ratio;
        let viewport_height = VIEWPORT_HEIGHT;

        let focal_length = 1.0;

        let camera_center = Vector3::zeros();

        let viewport_u = Vector3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vector3::new(0.0, -viewport_height, 0.0);

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left = camera_center
            - Vector3::new(0.0, 0.0, focal_length)
            - viewport_u / 2.0
            - viewport_v / 2.0;
        let pixel_origin = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            aspect_ratio,
            image_width,
            image_height,
            viewport_width,
            viewport_height,
            focal_length,
            camera_center,
            viewport_u,
            viewport_v,
            pixel_delta_u,
            pixel_delta_v,
            viewport_upper_left,
            pixel_origin,
        }
    }

    pub fn ray_color(&self, ray: &Ray) -> Color {
        let unit_direction = ray.direction.normalize();
        let a = 0.5 * (unit_direction[1] + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    pub fn render_image(&self) -> crate::image::Image {
        let mut image = crate::image::Image::new(self.image_width, self.image_height);

        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let pixel_center = self.pixel_origin
                    + (i as f64 * self.pixel_delta_u)
                    + (j as f64 * self.pixel_delta_v);
                let ray_direction = pixel_center - self.camera_center;

                let ray = Ray {
                    origin: self.camera_center,
                    direction: ray_direction,
                };

                let pixel_color = self.ray_color(&ray);

                *image.pixel(i, j) = crate::image::Color::from_float_vector(&pixel_color);
            }
        }

        image
    }
}
