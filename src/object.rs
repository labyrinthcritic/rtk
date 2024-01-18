use std::ops::Range;

use nalgebra::{UnitQuaternion, Vector3};

use crate::{material::Material, render::Ray};

pub struct World {
    pub objects: Vec<Object>,
    pub materials: Vec<Material>,
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
    Sphere {
        center: Vector3<f64>,
        radius: f64,
        material: usize,
    },
    Quad {
        /// The point from which the basis vectors extend.
        q: Vector3<f64>,
        /// First basis vector.
        u: Vector3<f64>,
        /// Second basis vector.
        v: Vector3<f64>,
        material: usize,
        /// Data calculated from the other parameters.
        cached: QuadCached,
    },
}

impl Object {
    pub fn sphere(center: Vector3<f64>, radius: f64, material: usize) -> Self {
        Self::Sphere {
            center,
            radius,
            material,
        }
    }

    pub fn quad(q: Vector3<f64>, u: Vector3<f64>, v: Vector3<f64>, material: usize) -> Self {
        let n = u.cross(&v);
        let normal = n.normalize();
        let d = normal.dot(&q);
        // TODO: possibly useless? normal vector is already normalized; normal == w
        let w = n / n.dot(&n);
        Self::Quad {
            q,
            u,
            v,
            material,
            cached: QuadCached { normal, d, w },
        }
    }

    /// Construct a prism.
    /// A prism is not a primitive object shape; it is 6 `Quad`s.
    pub fn prism(
        // The center of the bottom face.
        origin: &Vector3<f64>,
        width: f64,
        height: f64,
        depth: f64,
        rotation: &UnitQuaternion<f64>,
        material: usize,
    ) -> Vec<Self> {
        let u = rotation * (Vector3::new(1.0, 0.0, 0.0) * width);
        let v = rotation * (Vector3::new(0.0, 1.0, 0.0) * height);
        let w = rotation * (Vector3::new(0.0, 0.0, 1.0) * depth);

        let true_origin = origin - (u / 2.0) - (w / 2.0);
        let opposite_true_origin = origin + (u / 2.0) + (w / 2.0) + v;

        let quads = vec![
            Object::quad(true_origin, u, v, material),
            Object::quad(true_origin, v, w, material),
            Object::quad(true_origin, w, u, material),
            Object::quad(opposite_true_origin, -u, -v, material),
            Object::quad(opposite_true_origin, -v, -w, material),
            Object::quad(opposite_true_origin, -w, -u, material),
        ];

        quads
    }
}

pub struct QuadCached {
    normal: Vector3<f64>,
    d: f64,
    w: Vector3<f64>,
}

impl Object {
    pub fn hit(&self, ray: &Ray, ray_t: Range<f64>) -> Option<Hit> {
        match self {
            Object::Sphere {
                center: origin,
                radius,
                material,
            } => hit_sphere(ray, ray_t, origin, *radius, *material),
            Object::Quad {
                q,
                u,
                v,
                material,
                cached,
            } => hit_quad(ray, ray_t, q, u, v, *material, cached),
        }
    }
}

/// Data surrounding a ray intersecting with an object.
pub struct Hit {
    /// The point at which the ray intersected the object.
    pub p: Vector3<f64>,
    pub normal: Vector3<f64>,
    /// The time at which the intersection occurred.
    pub t: f64,
    /// Whether the normal points outward or inward.
    pub front_face: bool,
    /// The material of the struck object.
    pub material: usize,
}

/// Finds the time at which a ray will hit a sphere, or returns `None` if it will not.
fn hit_sphere(
    ray: &Ray,
    ray_t: Range<f64>,
    center: &Vector3<f64>,
    radius: f64,
    material: usize,
) -> Option<Hit> {
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
        material,
    })
}

fn hit_quad(
    ray: &Ray,
    ray_t: Range<f64>,
    q: &Vector3<f64>,
    u: &Vector3<f64>,
    v: &Vector3<f64>,
    material: usize,
    cache: &QuadCached,
) -> Option<Hit> {
    let denom = cache.normal.dot(&ray.direction);
    // if the ray is parallel to the plane, do not hit
    if denom.abs() < 1e-8 {
        return None;
    }

    let t = (cache.d - cache.normal.dot(&ray.origin)) / denom;
    if !ray_t.contains(&t) {
        return None;
    }

    let p = ray.at(t);
    let planar_hit = p - q;
    let alpha = cache.w.dot(&planar_hit.cross(v));
    let beta = cache.w.dot(&u.cross(&planar_hit));

    if !((0.0..1.0).contains(&alpha)) || !((0.0..1.0).contains(&beta)) {
        return None;
    }

    let (normal, front_face) = face_normal(ray, &cache.normal);

    Some(Hit {
        p,
        normal,
        t,
        front_face,
        material,
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
