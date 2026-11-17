mod render;
mod profile;

use clap::{Parser, Subcommand};

use profile::ProfileCommands;

use crate::cli::render::grayscale::{RenderGrayscaleArgs, render_grayscale_command};
use crate::cli::profile::profile_command;
use crate::cli::render::multiplex::{RenderMultiplexArgs, render_multiplex_command};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]    
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Render image file into disc data
    Render(RenderGrayscaleArgs),

    Multiplex(RenderMultiplexArgs),

    /// Profile operations
    #[command(subcommand)]    
    Profile(ProfileCommands),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Render(args) => render_grayscale_command(args),
        Command::Multiplex(args) => render_multiplex_command(args),
        Command::Profile(profile_commands) => profile_command(profile_commands),
    }
}