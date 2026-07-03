#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use anyhow::Result;
use clap::Parser;
use ripcanvas::{app, cli::Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let launch = cli.resolve_launch()?;
    app::run(launch)
}
