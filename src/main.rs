#![allow(unused)]

mod image;
mod object;
mod render;

use image::{Color, Image};
use nalgebra::Vector3;
use render::Renderer;

use crate::object::{Object, World};

fn main() {
    let width = 400;
    let height = 225;

    let renderer = Renderer::new(width, height);
    let world = World {
        objects: vec![
            Object::Sphere {
                origin: Vector3::new(0.0, 0.0, -1.0),
                radius: 0.5,
            },
            Object::Sphere {
                origin: Vector3::new(0.0, -100.5, -1.0),
                radius: 100.0,
            },
        ],
    };

    eprintln!("Rendering...");
    let image = renderer.render(&world);

    eprintln!("Writing to image.ppm...");
    std::fs::write("image.ppm", image.ppm().as_bytes()).unwrap();
}
