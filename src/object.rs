use std::ops::Range;

use nalgebra::Vector3;

use crate::render::Ray;

pub struct World {
    pub objects: Vec<Object>,
}

impl World {
    pub fn hit(&self, ray: &Ray, ray_t_min: f64, ray_t_max: f64) -> Option<Hit> {
        let mut hit = None;
        let mut closest = ray_t_max;

        for object in self.objects.iter() {
            if let Some(new_hit) = object.hit(ray, ray_t_min..closest) {
                closest = new_hit.t;
                hit = Some(new_hit);
            }
        }

        hit
    }
}

pub enum Object {
    Sphere { origin: Vector3<f64>, radius: f64 },
}

impl Object {
    pub fn hit(&self, ray: &Ray, ray_t: Range<f64>) -> Option<Hit> {
        match self {
            Object::Sphere { origin, radius } => hit_sphere(origin, *radius, ray, ray_t),
        }
    }
}

pub struct Hit {
    pub p: Vector3<f64>,
    pub normal: Vector3<f64>,
    pub t: f64,
    pub front_face: bool,
}

/// Finds the time at which a ray will hit a sphere, or returns `None` if it will not.
fn hit_sphere(center: &Vector3<f64>, radius: f64, ray: &Ray, ray_t: Range<f64>) -> Option<Hit> {
    // Quadratic formula
    let oc = ray.origin - center;
    let a = ray.direction.magnitude_squared();
    let half_b = oc.dot(&ray.direction);
    let c = oc.magnitude_squared() - radius.powi(2);
    let discriminant = half_b.powi(2) - a * c;

    // If the discriminant is less than zero, there are no solutions.
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();

    // Find the nearest root in the ray_t range.
    let mut root = (-half_b - sqrt_d) / a;
    if !ray_t.contains(&root) {
        root = (-half_b + sqrt_d) / a;
        if !ray_t.contains(&root) {
            return None;
        }
    }

    let t = root;
    let p = ray.at(t);
    let outward_normal = (p - center) / radius;
    let (normal, front_face) = face_normal(ray, &outward_normal);

    Some(Hit {
        p,
        normal,
        t,
        front_face,
    })
}

/// Determine the normal vector for a hit. Returns a tuple of the normal and if the normal is front-facing (outward).
/// `outward_normal` must have unit length.
fn face_normal(ray: &Ray, outward_normal: &Vector3<f64>) -> (Vector3<f64>, bool) {
    let front_face = ray.direction.dot(outward_normal) < 0.0;
    let normal = if front_face {
        *outward_normal
    } else {
        -outward_normal
    };

    (normal, front_face)
}
