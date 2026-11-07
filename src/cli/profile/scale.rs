use std::io::Write;
use std::fs::File;

use anyhow::Context;
use clap::Args;
use discscape::profile::{CorrectionProfile, DiscProfile};

#[derive(Args)]
pub struct ProfileScaleArgs {
    /// Path to the input disc profile document
    profile: String,
    /// Path to the output profile. Use '-' for output to stdout
    output: String,
    /// Scale factor
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

    let inner: Box<dyn Write> = if &args.output == "-" {
        Box::new(std::io::stdout().lock())
    } else {
        let file = File::create_new(&args.output)
            .with_context(|| format!("Failed to create or open output file '{}'", args.output))?;
        Box::new(file)
    };
    serde_json::to_writer_pretty(inner, &profile)
        .with_context(|| format!("Failed to profile to file '{}'", args.output))?;

    Ok(())
}