//! This module describes the model of a scene file.

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub camera: Camera,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Camera {
    pub image_dimensions: (u32, u32),
    pub position: Option<(f64, f64, f64)>,
    pub rotation: Option<Rotation>,
    pub fov: f64,
    pub defocus: Option<Defocus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Rotation {
    Euler { roll: f64, pitch: f64, yaw: f64 },
    Direction { x: f64, y: f64, z: f64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Defocus {
    pub focus_distance: f64,
    pub defocus_angle: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Material {
    Diffuse { albedo: (f64, f64, f64) },
    Metal { albedo: (f64, f64, f64) },
    Dielectric { ir: f64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Object {
    pub material: usize,
    pub shape: Shape,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Shape {
    Sphere {
        center: (f64, f64, f64),
        radius: f64,
    },
}

impl From<Material> for crate::material::Material {
    fn from(value: Material) -> Self {
        match value {
            Material::Diffuse { albedo } => crate::material::Material::Diffuse {
                albedo: Vector3::new(albedo.0, albedo.1, albedo.2),
            },
            Material::Metal { albedo } => crate::material::Material::Metal {
                albedo: Vector3::new(albedo.0, albedo.1, albedo.2),
            },
            Material::Dielectric { ir } => crate::material::Material::Dielectric { ir },
        }
    }
}
