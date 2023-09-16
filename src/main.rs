mod image;
mod material;
mod object;
mod render;

use std::{rc::Rc, thread};

use image::Image;
use nalgebra::{Unit, UnitQuaternion, Vector3};
use render::Renderer;

use crate::{
    material::Material,
    object::{Object, World},
    render::Camera,
};

fn main() {
    let width = 640;
    let height = 360;

    let from = Vector3::new(-2.0, 2.0, 1.0);
    let to = Vector3::new(0.0, 0.0, -1.0);

    let rotation = UnitQuaternion::rotation_between(&-Vector3::z(), &(to - from)).unwrap();

    let camera = Camera {
        position: Vector3::new(-2.0, 2.0, 1.0),
        rotation,
        fov: 20.0,
        focus_distance: 3.4,
        defocus_angle: 10.0,
    };

    let (renderer, progress_receiver) = Renderer::new(width, height, camera);

    let handle = thread::spawn(|| render(renderer));

    loop {
        let progress = progress_receiver.recv().unwrap();
        print_progress_bar(progress);
        if progress == 100 {
            break;
        }
    }

    let image = handle.join().unwrap();

    eprintln!("\nWriting to image.ppm...");
    std::fs::write("image.ppm", image.ppm().as_bytes()).unwrap();
}

fn render(renderer: Renderer) -> Image {
    let material_ground = Material::diffuse(0.8, 0.8, 0.0);
    let material_center = Material::diffuse(0.1, 0.2, 0.5);
    let material_left = Material::dielectric(1.5);
    let material_right = Material::metal(0.8, 0.6, 0.2);

    let world = World {
        objects: vec![
            Object::Sphere {
                origin: Vector3::new(0.0, -100.5, -1.0),
                radius: 100.0,
                material: Rc::new(material_ground),
            },
            Object::Sphere {
                origin: Vector3::new(0.0, 0.0, -1.0),
                radius: 0.5,
                material: Rc::new(material_center),
            },
            Object::Sphere {
                origin: Vector3::new(-1.0, 0.0, -1.0),
                radius: 0.5,
                material: Rc::new(material_left),
            },
            Object::Sphere {
                origin: Vector3::new(1.0, 0.0, -1.0),
                radius: 0.5,
                material: Rc::new(material_right),
            },
        ],
    };

    eprintln!("Rendering...");
    renderer.render(&world)
}

fn print_progress_bar(progress: u32) {
    const SEGMENTS: u32 = 40;

    let filled_segments = ((progress as f32 / 100.0) * SEGMENTS as f32) as u32;
    let empty_segments = SEGMENTS - filled_segments;

    eprint!("\r[");
    for _ in 0..filled_segments {
        eprint!("*");
    }
    for _ in 0..empty_segments {
        eprint!(" ");
    }
    eprint!("] {progress}%");
}
