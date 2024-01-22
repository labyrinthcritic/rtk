mod cli;
#[cfg(feature = "denoise")]
mod denoise;
mod material;
mod object;
mod render;
mod scene;

use std::{path::Path, thread};

use colored::Colorize;
use nalgebra::{Unit, UnitQuaternion, Vector3};

use crate::{
    material::Material,
    object::{Object, World},
    render::{Camera, Renderer},
    scene::Scene,
};

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("{}: {}", "error".bold().red(), e);
        std::process::exit(1);
    }
}

fn run_cli() -> anyhow::Result<()> {
    let cli = <cli::Cli as clap::Parser>::parse();

    match cli.command {
        cli::Command::Render {
            scene,
            output,
            parallel: _,
            no_parallel,
            #[cfg(feature = "denoise")]
            denoise,
        } => {
            #[cfg(feature = "denoise")]
            render(scene.as_path(), output.as_path(), !no_parallel, denoise)?;
            #[cfg(not(feature = "denoise"))]
            render(scene.as_path(), output.as_path(), !no_parallel, false)?;
        }
        #[cfg(feature = "denoise")]
        cli::Command::Denoise { image, output } => denoise(&image, output.as_deref()),
    }

    Ok(())
}

/// Handle `cli::Command::Render`.
fn render(
    scene_path: &Path,
    output_path: &Path,
    parallel: bool,
    _denoise: bool,
) -> anyhow::Result<()> {
    let scene_source = std::fs::read_to_string(scene_path)?;
    let scene: Scene = toml::from_str(&scene_source)?;

    let camera = create_camera(&scene);
    let (renderer, progress_receiver) = Renderer::new(camera);

    let handle = thread::spawn(move || {
        let materials = collect_materials(&scene);
        let objects = create_objects(&scene);

        renderer.render(&World { objects, materials }, parallel)
    });

    loop {
        let progress = progress_receiver.recv()?;
        print_progress_bar(progress);
        if progress == 100 {
            eprintln!();
            break;
        }
    }

    let image = handle
        .join()
        .map_err(|e| anyhow::anyhow!("the rendering thread panicked:\n{:#?}", e))?;

    #[cfg(feature = "denoise")]
    let image = if _denoise {
        eprintln!("Denoising...");
        denoise::denoise(&image)
    } else {
        image
    };

    eprintln!("Writing to {}...", output_path.display());
    image.save(output_path)?;

    Ok(())
}

#[cfg(feature = "denoise")]
/// Handle `cli::Command::Denoise`.
fn denoise(image_path: &Path, output_path: Option<&Path>) -> anyhow::Result<()> {
    let image = image::io::Reader::open(image_path)?.decode()?.to_rgb8();
    eprintln!("Denoising {}...", image_path.display());
    let denoised = denoise::denoise(&image);
    denoised.save(output_path.unwrap_or(image_path))?;

    Ok(())
}

fn collect_materials(scene: &Scene) -> Vec<Material> {
    let mut result = Vec::new();
    for m in scene.materials.iter() {
        result.push(<scene::Material as Into<Material>>::into(m.clone()));
    }

    result
}

fn create_objects(scene: &Scene) -> Vec<Object> {
    let mut result = vec![];

    for obj in scene.objects.iter() {
        match obj.shape {
            scene::Shape::Sphere { center, radius } => result.push(Object::sphere(
                tuple_to_vector(center),
                radius,
                obj.material,
            )),
            scene::Shape::Quad { q, u, v } => result.push(Object::quad(
                tuple_to_vector(q),
                tuple_to_vector(u),
                tuple_to_vector(v),
                obj.material,
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
                obj.material,
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
        samples_per_pixel: scene.camera.samples_per_pixel.unwrap_or(100),
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
