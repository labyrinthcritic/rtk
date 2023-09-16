mod cli;
mod image;
mod material;
mod object;
mod render;
mod scene;

use std::{rc::Rc, thread};

use nalgebra::{UnitQuaternion, Vector3};
use render::Renderer;

use crate::{
    material::Material,
    object::{Object, World},
    render::Camera,
    scene::Scene,
};

fn main() {
    let cli = <cli::Cli as clap::Parser>::parse();

    let scene_path = cli.scene;
    let scene_source = std::fs::read_to_string(scene_path).unwrap();
    let scene: Scene = toml::from_str(&scene_source).unwrap();

    let camera = create_camera(&scene);
    let (renderer, progress_receiver) = Renderer::new(camera);

    let handle = thread::spawn(move || {
        let materials = collect_materials(&scene);
        let objects = create_objects(&scene, materials.as_slice());

        renderer.render(&World { objects })
    });

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

fn collect_materials(scene: &Scene) -> Vec<Rc<Material>> {
    let mut result = Vec::new();
    for m in scene.materials.iter() {
        result.push(Rc::new(<scene::Material as Into<Material>>::into(
            m.clone(),
        )));
    }

    result
}

fn create_objects(scene: &Scene, materials: &[Rc<Material>]) -> Vec<Object> {
    scene
        .objects
        .iter()
        .map(|obj| match obj.shape {
            scene::Shape::Sphere { center, radius } => Object::Sphere {
                center: Vector3::new(center.0, center.1, center.2),
                radius,
                material: Rc::clone(&materials[obj.material]),
            },
        })
        .collect()
}

fn create_camera(scene: &Scene) -> Camera {
    let p = scene.camera.position;

    let rotation = match scene.camera.rotation {
        scene::Rotation::Euler { roll, pitch, yaw } => {
            UnitQuaternion::from_euler_angles(roll, pitch, yaw)
        }
        scene::Rotation::Direction { x, y, z } => {
            UnitQuaternion::rotation_between(&-Vector3::z(), &Vector3::new(x, y, z)).unwrap()
        }
    };

    let (focus_distance, defocus_angle) = if let Some(defocus) = &scene.camera.defocus {
        (defocus.focus_distance, defocus.defocus_angle)
    } else {
        (1.0, 0.0)
    };

    Camera {
        image_width: scene.camera.image_dimensions.0,
        image_height: scene.camera.image_dimensions.1,
        position: Vector3::new(p.0, p.1, p.2),
        rotation,
        fov: scene.camera.fov,
        focus_distance,
        defocus_angle,
    }
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
