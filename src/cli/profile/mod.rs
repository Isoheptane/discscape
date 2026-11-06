mod show;
mod scale;

use clap::Subcommand;

use crate::cli::profile::show::{ProfileShowArgs, profile_show_command};
use crate::cli::profile::scale::{ProfileScaleArgs, profile_scale_command};

#[derive(Subcommand)]
pub enum ProfileCommands {
    Show(ProfileShowArgs),
    Scale(ProfileScaleArgs),
}

pub fn profile_command(cmd: ProfileCommands) -> anyhow::Result<()> {
    match cmd {
        ProfileCommands::Show(args) => {
            profile_show_command(args)
        },
        ProfileCommands::Scale(args) => {
            profile_scale_command(args)
        },
    }
}