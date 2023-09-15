use std::{ops::Range, sync::mpsc};

use nalgebra::{constraint::SameNumberOfColumns, UnitQuaternion, Vector3};
use rand::Rng;

use crate::object::World;

pub struct Ray {
    pub origin: Vector3<f64>,
    pub direction: Vector3<f64>,
}

impl Ray {
    pub fn at(&self, t: f64) -> Vector3<f64> {
        self.origin + t * self.direction
    }
}

pub type Color = Vector3<f64>;

#[derive(Clone, Debug, Default)]
pub struct Camera {
    pub position: Vector3<f64>,
    pub rotation: UnitQuaternion<f64>,
    pub fov: f64,
}

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
    samples_per_pixel: u32,
    max_ray_bounces: u32,
    progress_sender: mpsc::Sender<u32>,
}

impl Renderer {
    pub fn new(image_width: u32, image_height: u32, camera: Camera) -> (Self, mpsc::Receiver<u32>) {
        let aspect_ratio = image_width as f64 / image_height as f64;

        let focal_length = 1.0;

        let viewport_height = 2.0 * focal_length * (camera.fov.to_radians() / 2.0).tan();
        let viewport_width = viewport_height * aspect_ratio;

        let camera_center = camera.position;

        // camera +x unit vector
        let u = camera.rotation * Vector3::x();
        let v = camera.rotation * Vector3::y();
        let w = camera.rotation * Vector3::z();

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            camera_center - (focal_length * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_origin = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let (sender, receiver) = mpsc::channel();

        (
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
                samples_per_pixel: 100,
                max_ray_bounces: 50,
                progress_sender: sender,
            },
            receiver,
        )
    }

    /// Get the precise color of any ray in the world.
    pub fn ray_color(&self, world: &World, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::zeros();
        }

        if let Some(hit) = world.hit(ray, 0.001, f64::INFINITY) {
            if let Some((attenuation, scattered)) = hit.material.scatter(ray, &hit) {
                return attenuation.component_mul(&self.ray_color(world, &scattered, depth - 1));
            }
            return Color::zeros();
        }

        let unit_direction = ray.direction.normalize();
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    /// Render a complete world, casting several rays for each pixel and collecting them into a complete image.
    pub fn render(&self, world: &World) -> crate::image::Image {
        let mut image = crate::image::Image::new(self.image_width, self.image_height);

        let total_pixels = self.image_width * self.image_height;
        let mut pixels_completed = 0;
        let mut progress = 0;

        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let pixel_center = self.pixel_origin
                    + (i as f64 * self.pixel_delta_u)
                    + (j as f64 * self.pixel_delta_v);
                let ray_direction = pixel_center - self.camera_center;

                let mut pixel_color = Color::zeros();

                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    pixel_color += self.ray_color(world, &ray, self.max_ray_bounces);
                }

                // Divide to compute the average color between all samples
                pixel_color /= self.samples_per_pixel as f64;
                // Gamma correct
                pixel_color = linear_to_gamma(&pixel_color);

                *image.pixel(i, j) = crate::image::Color::from_float_vector(&pixel_color);

                pixels_completed += 1;

                if (pixels_completed * 100 / total_pixels) > progress {
                    progress = pixels_completed * 100 / total_pixels;
                    self.progress_sender.send(progress);
                }
            }
        }

        image
    }

    /// Get a randomly sampled camera ray for the pixel at location (i, j).
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let pixel_center =
            self.pixel_origin + (i as f64 * self.pixel_delta_u) + (j as f64 * self.pixel_delta_v);
        let pixel_sample = pixel_center + self.pixel_sample_square();

        let ray_direction = pixel_sample - self.camera_center;

        Ray {
            origin: self.camera_center,
            direction: ray_direction,
        }
    }

    /// Get a random location within the size of a pixel on the viewport.
    fn pixel_sample_square(&self) -> Vector3<f64> {
        let mut thread_rng = rand::thread_rng();
        let px = -0.5 + thread_rng.gen_range(0.0..1.0);
        let py = -0.5 + thread_rng.gen_range(0.0..1.0);

        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }
}

fn linear_to_gamma(linear_color: &Vector3<f64>) -> Vector3<f64> {
    Vector3::new(
        linear_color.x.sqrt(),
        linear_color.y.sqrt(),
        linear_color.z.sqrt(),
    )
}

pub fn vector_near_zero(v: &Vector3<f64>) -> bool {
    const S: f64 = 1e-8;
    (v.x.abs() < S) && (v.y.abs() < S) && (v.z.abs() < S)
}

pub fn random_vector() -> Vector3<f64> {
    let mut thread_rng = rand::thread_rng();
    Vector3::new(
        thread_rng.gen_range(0.0..1.0),
        thread_rng.gen_range(0.0..1.0),
        thread_rng.gen_range(0.0..1.0),
    )
}

pub fn random_vector_range(range: Range<f64>) -> Vector3<f64> {
    let mut thread_rng = rand::thread_rng();
    Vector3::new(
        thread_rng.gen_range(range.clone()),
        thread_rng.gen_range(range.clone()),
        thread_rng.gen_range(range),
    )
}

pub fn random_vector_in_unit_sphere() -> Vector3<f64> {
    loop {
        let vec = random_vector_range(-1.0..1.0);
        if vec.magnitude_squared() <= 1.0 {
            return vec;
        }
    }
}

pub fn random_unit_vector() -> Vector3<f64> {
    random_vector_in_unit_sphere().normalize()
}

pub fn random_unit_vector_on_hemisphere(normal: &Vector3<f64>) -> Vector3<f64> {
    let vec = random_unit_vector();
    if vec.dot(normal) > 0.0 {
        vec
    } else {
        -vec
    }
}
