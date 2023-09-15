use std::ops::Range;

use nalgebra::{constraint::SameNumberOfColumns, Vector3};
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
            samples_per_pixel: 100,
            max_ray_bounces: 50,
        }
    }

    pub fn ray_color(&self, world: &World, ray: &Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::zeros();
        }

        if let Some(hit) = world.hit(ray, 0.001, f64::INFINITY) {
            let new_direction = random_unit_vector_on_hemisphere(&hit.normal);
            return 0.5
                * self.ray_color(
                    world,
                    &Ray {
                        origin: hit.p,
                        direction: new_direction,
                    },
                    depth - 1,
                );
        }

        let unit_direction = ray.direction.normalize();
        let a = 0.5 * (unit_direction[1] + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    pub fn render(&self, world: &World) -> crate::image::Image {
        let mut image = crate::image::Image::new(self.image_width, self.image_height);

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

                pixel_color /= self.samples_per_pixel as f64;
                *image.pixel(i, j) = crate::image::Color::from_float_vector(&pixel_color);
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

fn random_vector() -> Vector3<f64> {
    let mut thread_rng = rand::thread_rng();
    Vector3::new(
        thread_rng.gen_range(0.0..1.0),
        thread_rng.gen_range(0.0..1.0),
        thread_rng.gen_range(0.0..1.0),
    )
}

fn random_vector_range(range: Range<f64>) -> Vector3<f64> {
    let mut thread_rng = rand::thread_rng();
    Vector3::new(
        thread_rng.gen_range(range.clone()),
        thread_rng.gen_range(range.clone()),
        thread_rng.gen_range(range),
    )
}

fn random_vector_in_unit_sphere() -> Vector3<f64> {
    loop {
        let vec = random_vector_range(-1.0..1.0);
        if vec.magnitude_squared() <= 1.0 {
            return vec;
        }
    }
}

fn random_unit_vector() -> Vector3<f64> {
    random_vector_in_unit_sphere().normalize()
}

fn random_unit_vector_on_hemisphere(normal: &Vector3<f64>) -> Vector3<f64> {
    let vec = random_unit_vector();
    if vec.dot(normal) > 0.0 {
        vec
    } else {
        -vec
    }
}
