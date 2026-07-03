#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use anyhow::Result;
use clap::Parser;
use ripcanvas::{
    app,
    cli::{Cli, CliAction},
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.run()? {
        CliAction::Launch(launch) => app::run(launch),
        CliAction::Export { input, output } => {
            println!("Exported {} to {}", input.display(), output.display());
            Ok(())
        }
    }
}
