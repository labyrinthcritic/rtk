use std::{ops::Range, sync::mpsc};

use nalgebra::{UnitQuaternion, Vector3};
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
    pub image_width: u32,
    pub image_height: u32,
    pub position: Vector3<f64>,
    pub rotation: UnitQuaternion<f64>,
    pub fov: f64,
    pub focus_distance: f64,
    pub defocus_angle: f64,
}

pub struct Renderer {
    samples_per_pixel: u32,
    max_ray_bounces: u32,
    progress_sender: mpsc::Sender<u32>,

    // values computed from camera and viewport
    /// In pixels.
    image_width: u32,
    /// In pixels.
    image_height: u32,
    camera_center: Vector3<f64>,
    pixel_delta_u: Vector3<f64>,
    pixel_delta_v: Vector3<f64>,
    pixel_origin: Vector3<f64>,
    defocus_angle: f64,
    defocus_disk_u: Vector3<f64>,
    defocus_disk_v: Vector3<f64>,
}

impl Renderer {
    pub fn new(camera: Camera) -> (Self, mpsc::Receiver<u32>) {
        let aspect_ratio = camera.image_width as f64 / camera.image_height as f64;

        let viewport_height = 2.0 * camera.focus_distance * (camera.fov.to_radians() / 2.0).tan();
        let viewport_width = viewport_height * aspect_ratio;

        let camera_center = camera.position;

        // camera +x unit vector
        let u = camera.rotation * Vector3::x();
        let v = camera.rotation * Vector3::y();
        let w = camera.rotation * Vector3::z();

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / camera.image_width as f64;
        let pixel_delta_v = viewport_v / camera.image_height as f64;

        let viewport_upper_left =
            camera_center - (camera.focus_distance * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_origin = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius =
            camera.focus_distance * (camera.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let (sender, receiver) = mpsc::channel();

        (
            Self {
                image_width: camera.image_width,
                image_height: camera.image_height,
                camera_center,
                pixel_delta_u,
                pixel_delta_v,
                pixel_origin,
                samples_per_pixel: 100,
                max_ray_bounces: 50,
                progress_sender: sender,
                defocus_angle: camera.defocus_angle,
                defocus_disk_u,
                defocus_disk_v,
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
    pub fn render(&self, world: &World) -> image::RgbImage {
        let mut image = image::RgbImage::new(self.image_width, self.image_height);

        let total_pixels = self.image_width * self.image_height;
        let mut pixels_completed = 0;
        let mut progress = 0;

        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let mut pixel_color = Color::zeros();

                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    pixel_color += self.ray_color(world, &ray, self.max_ray_bounces);
                }

                // Divide to compute the average color between all samples
                pixel_color /= self.samples_per_pixel as f64;
                // Gamma correct
                pixel_color = linear_to_gamma(&pixel_color);
                let rgb = color_to_rgb(&pixel_color);

                image.put_pixel(i, j, image::Rgb(rgb));

                pixels_completed += 1;

                if (pixels_completed * 100 / total_pixels) > progress {
                    progress = pixels_completed * 100 / total_pixels;
                    self.progress_sender.send(progress).unwrap();
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

        let origin = if self.defocus_angle <= 0.0 {
            self.camera_center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - origin;

        Ray {
            origin,
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

    fn defocus_disk_sample(&self) -> Vector3<f64> {
        let p = random_vector_in_unit_disk();
        self.camera_center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
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

pub fn random_vector_in_unit_disk() -> Vector3<f64> {
    let mut thread_rng = rand::thread_rng();
    loop {
        let vec = Vector3::new(
            thread_rng.gen_range(-1.0..1.0),
            thread_rng.gen_range(-1.0..1.0),
            0.0,
        );
        if vec.magnitude_squared() <= 1.0 {
            return vec;
        }
    }
}

fn color_to_rgb(c: &Vector3<f64>) -> [u8; 3] {
    [
        (c.x * 255.999) as u8,
        (c.y * 255.999) as u8,
        (c.z * 255.999) as u8,
    ]
}
