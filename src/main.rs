mod cli;
mod material;
mod object;
mod render;
mod scene;

use std::{rc::Rc, thread};

use nalgebra::{Unit, UnitQuaternion, Vector3};
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

    eprintln!("\nWriting to {}...", cli.output.to_string_lossy());
    image.save(cli.output).unwrap();
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
    let mut result = vec![];

    for obj in scene.objects.iter() {
        match obj.shape {
            scene::Shape::Sphere { center, radius } => result.push(Object::sphere(
                tuple_to_vector(center),
                radius,
                Rc::clone(&materials[obj.material]),
            )),
            scene::Shape::Quad { q, u, v } => result.push(Object::quad(
                tuple_to_vector(q),
                tuple_to_vector(u),
                tuple_to_vector(v),
                Rc::clone(&materials[obj.material]),
            )),
            scene::Shape::Prism {
                origin,
                width,
                height,
                depth,
                ref rotation,
            } => result.extend(Object::prism(
                &tuple_to_vector(origin),
                width,
                height,
                depth,
                &rotation.clone().unwrap_or_default().into(),
                Rc::clone(&materials[obj.material]),
            )),
        }
    }

    result
}

fn create_camera(scene: &Scene) -> Camera {
    let p = scene.camera.position.unwrap_or_default();

    let rotation = if let Some(rotation) = &scene.camera.rotation {
        match rotation {
            scene::Rotation::Euler { roll, pitch, yaw } => {
                UnitQuaternion::from_euler_angles(*roll, *pitch, *yaw)
            }
            scene::Rotation::Direction { x, y, z } => {
                UnitQuaternion::rotation_between(&-Vector3::z(), &Vector3::new(*x, *y, *z))
                    .unwrap_or_else(|| {
                        UnitQuaternion::from_axis_angle(
                            &Unit::new_normalize(Vector3::y()),
                            std::f64::consts::PI,
                        )
                    })
            }
        }
    } else {
        UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0)
    };

    let (focus_distance, defocus_angle) = if let Some(defocus) = &scene.camera.defocus {
        (defocus.focus_distance, defocus.defocus_angle)
    } else {
        (1.0, 0.0)
    };

    let background_color = scene.camera.background_color.unwrap_or_default();

    Camera {
        image_width: scene.camera.image_dimensions.0,
        background_color: tuple_to_vector(background_color),
        image_height: scene.camera.image_dimensions.1,
        position: tuple_to_vector(p),
        rotation,
        fov: scene.camera.fov,
        focus_distance,
        defocus_angle,
    }
}

fn tuple_to_vector((x, y, z): (f64, f64, f64)) -> Vector3<f64> {
    Vector3::new(x, y, z)
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
