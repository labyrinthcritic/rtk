use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Render a scene.
    Render {
        /// Path to the scene to render.
        scene: PathBuf,
        /// Path to write the output image to.
        #[arg(short, long, default_value = "image.png")]
        output: PathBuf,
        /// Render across all available CPU threads [default].
        #[arg(long, group = "parallel_option")]
        parallel: bool,
        /// Render only on the main thread.
        #[arg(long, group = "parallel_option")]
        no_parallel: bool,
        #[cfg(feature = "denoise")]
        /// Denoise the image after rendering.
        #[arg(short, long)]
        denoise: bool,
    },
    #[cfg(feature = "denoise")]
    /// Denoise an image.
    Denoise {
        /// Path to the image to denoise.
        image: PathBuf,
        /// Path to write the output image to (overwrites input by default).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
