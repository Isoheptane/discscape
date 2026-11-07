use std::{fs::File, io::Write};

use anyhow::Context;
use clap::Args;
use discscape::profile::{CorrectionProfile, DiscProfile};

#[derive(Args)]
pub struct ProfileScaleArgs {
    profile: String,
    output: String,
    factor: f64
}

pub fn profile_scale_command(args: ProfileScaleArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("Failed to read profile file at '{}'", args.profile))?;
    let mut profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.profile))?;

    let factor = args.factor;

    profile.geometry.r0_mm *= factor;
    profile.geometry.pitch_um *= factor;

    if let Some(correction) = &mut profile.correction {
        match correction {
            CorrectionProfile::Interpolation(interpo) => {
                for sample in interpo.samples.iter_mut() {
                    sample[0] *= factor;
                }
            }
        }
    }

    let profile_content = serde_json::to_string_pretty(&profile)?;
    let mut file = File::create_new(&args.output)
        .with_context(|| format!("Failed to open profile file '{}'", args.output))?;
    file.write_all(profile_content.as_bytes())
        .with_context(|| format!("Failed to write profile file '{}'", args.output))?;

    Ok(())
}