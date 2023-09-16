use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the scene to render.
    pub scene: PathBuf,
}
