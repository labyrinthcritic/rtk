use nalgebra::Vector3;
use rand::Rng;

use crate::{
    object::Hit,
    render::{random_unit_vector, vector_near_zero, Color, Ray},
};

pub enum Material {
    Diffuse {
        albedo: Color,
    },
    Metal {
        albedo: Color,
    },
    Dielectric {
        /// Index of refraction.
        ir: f64,
    },
    Light {
        color: Color,
    },
}

#[allow(unused)]
impl Material {
    pub fn diffuse(r: f64, g: f64, b: f64) -> Self {
        Self::Diffuse {
            albedo: Vector3::new(r, g, b),
        }
    }

    pub fn metal(r: f64, g: f64, b: f64) -> Self {
        Self::Metal {
            albedo: Vector3::new(r, g, b),
        }
    }

    pub fn dielectric(ir: f64) -> Self {
        Self::Dielectric { ir }
    }

    /// Scatter a ray according to this material.
    pub fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<(Color, Ray)> {
        match self {
            Material::Diffuse { albedo } => scatter_diffuse(ray, hit, albedo),
            Material::Metal { albedo } => scatter_metal(ray, hit, albedo),
            Material::Dielectric { ir } => scatter_dielectric(ray, hit, *ir),
            Material::Light { color } => None,
        }
    }

    pub fn emit(&self) -> Color {
        match self {
            Material::Diffuse { albedo } => Color::zeros(),
            Material::Metal { albedo } => Color::zeros(),
            Material::Dielectric { ir } => Color::zeros(),
            Material::Light { color } => *color,
        }
    }
}

fn scatter_diffuse(_ray: &Ray, hit: &Hit, albedo: &Color) -> Option<(Color, Ray)> {
    let mut scatter_direction = hit.normal + random_unit_vector();

    // If the scatter direction is too small, it can cause floating point issues
    if vector_near_zero(&scatter_direction) {
        scatter_direction = hit.normal;
    }

    Some((
        *albedo,
        Ray {
            origin: hit.p,
            direction: scatter_direction,
        },
    ))
}

fn scatter_metal(ray: &Ray, hit: &Hit, albedo: &Color) -> Option<(Color, Ray)> {
    let reflected = reflect(&ray.direction.normalize(), &hit.normal);
    let scattered = Ray {
        origin: hit.p,
        direction: reflected,
    };
    let attenuation = *albedo;

    Some((attenuation, scattered))
}

fn scatter_dielectric(ray: &Ray, hit: &Hit, ir: f64) -> Option<(Color, Ray)> {
    let attenuation = Vector3::new(1.0, 1.0, 1.0);
    let refraction_ratio = if hit.front_face { 1.0 / ir } else { ir };

    let unit_direction = ray.direction.normalize();
    let cos_theta = (-unit_direction).dot(&hit.normal).min(1.0);
    let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    let direction = if cannot_refract
        || reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen_range(0.0..1.0)
    {
        reflect(&unit_direction, &hit.normal)
    } else {
        refract(&unit_direction, &hit.normal, refraction_ratio)
    };

    let scattered = Ray {
        origin: hit.p,
        direction,
    };

    Some((attenuation, scattered))
}

/// Reflect a vector `v` along a normal `n`.
fn reflect(v: &Vector3<f64>, n: &Vector3<f64>) -> Vector3<f64> {
    v - 2.0 * v.dot(n) * n
}

/// Refract a vector `uv` along a surface, according to Snell's law.
fn refract(uv: &Vector3<f64>, n: &Vector3<f64>, etai_over_etat: f64) -> Vector3<f64> {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -((1.0 - r_out_perp.magnitude_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}

/// Schlick's approximation of reflectance.
fn reflectance(cosine: f64, refraction_ratio: f64) -> f64 {
    let mut r0 = (1.0 - refraction_ratio) / (1.0 + refraction_ratio);
    r0 *= r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
