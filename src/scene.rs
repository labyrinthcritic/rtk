//! This module describes the model of a scene file.

use nalgebra::{UnitQuaternion, Vector3};
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
    pub background_color: Option<(f64, f64, f64)>,
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
    Diffuse {
        albedo: (f64, f64, f64),
    },
    Metal {
        albedo: (f64, f64, f64),
    },
    Dielectric {
        /// Index of refraction.
        ir: f64,
    },
    Light {
        color: (f64, f64, f64),
    },
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
    Quad {
        q: (f64, f64, f64),
        u: (f64, f64, f64),
        v: (f64, f64, f64),
    },
    Prism {
        /// The center of the bottom face.
        origin: (f64, f64, f64),
        width: f64,
        height: f64,
        depth: f64,
        rotation: Option<Rotation>,
    },
}

impl Default for Rotation {
    fn default() -> Self {
        Self::Euler {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

impl From<Rotation> for UnitQuaternion<f64> {
    fn from(rotation: Rotation) -> Self {
        match rotation {
            Rotation::Euler { roll, pitch, yaw } => {
                UnitQuaternion::from_euler_angles(roll, pitch, yaw)
            }
            Rotation::Direction { x, y, z } => {
                UnitQuaternion::rotation_between(&-Vector3::z(), &Vector3::new(x, y, z)).unwrap()
            }
        }
    }
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
            Material::Light { color } => crate::material::Material::Light {
                color: Vector3::new(color.0, color.1, color.2),
            },
        }
    }
}
