mod render;
mod profile;

use clap::{Parser, Subcommand};

use profile::ProfileCommands;

use crate::cli::{profile::profile_command, render::{RenderArgs, render_command}};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]    
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Render(RenderArgs),
    #[command(subcommand)]    
    Profile(ProfileCommands),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Render(args) => render_command(args),
        Command::Profile(profile_commands) => profile_command(profile_commands),
    }
}