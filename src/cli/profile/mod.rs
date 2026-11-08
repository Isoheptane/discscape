mod show;
mod scale;
mod rebase;
mod fit;

use clap::Subcommand;

use crate::cli::profile::fit::{ProfileFitArgs, profile_fit_command};
use crate::cli::profile::rebase::{ProfileRebaseArgs, profile_rebase_command};
use crate::cli::profile::show::{ProfileShowArgs, profile_show_command};
use crate::cli::profile::scale::{ProfileScaleArgs, profile_scale_command};

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// Inspect profile content
    Show(ProfileShowArgs),
    /// Scale profile geometry size
    Scale(ProfileScaleArgs),
    /// Rebase existing corrected profile
    Rebase(ProfileRebaseArgs),
    /// Fit profile to error measurement
    Fit(ProfileFitArgs),
}

pub fn profile_command(cmd: ProfileCommands) -> anyhow::Result<()> {
    match cmd {
        ProfileCommands::Show(args) => {
            profile_show_command(args)
        },
        ProfileCommands::Scale(args) => {
            profile_scale_command(args)
        },
        ProfileCommands::Rebase(args) => {
            profile_rebase_command(args)
        },
        ProfileCommands::Fit(args) => {
            profile_fit_command(args)
        },
    }
}