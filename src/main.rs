#![allow(unused)]

mod image;
mod render;

use image::{Color, Image};
use render::Renderer;

fn main() {
    let width = 400;
    let height = 225;

    let renderer = Renderer::new(width, height);
    eprintln!("Rendering...");
    let image = renderer.render_image();

    eprintln!("Writing to image.ppm...");
    std::fs::write("image.ppm", image.ppm().as_bytes()).unwrap();
}
