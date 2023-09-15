#![allow(unused)]

mod image;
mod object;
mod render;

use std::thread;

use image::{Color, Image};
use nalgebra::Vector3;
use render::Renderer;

use crate::object::{Object, World};

fn main() {
    let width = 400;
    let height = 225;

    let (renderer, progress_receiver) = Renderer::new(width, height);

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
    image
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
