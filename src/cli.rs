use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the scene to render.
    pub scene: PathBuf,
    /// Path to write the output image to.
    #[arg(short, long, default_value = "image.png")]
    pub output: PathBuf,
}
