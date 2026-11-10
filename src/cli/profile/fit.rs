use std::fs::File;
use std::io::Write;

use anyhow::Context;
use clap::Args;
use discscape::geometry::correction::{ExtrapolationMethod, InterpolationMethod, PhaseCorrection, PhaseCorrectionInterpolation};
use discscape::measure::MeasurementDoc;
use discscape::profile::{CorrectionProfile, DiscProfile, GeometryProfile};
use discscape::geometry::{PhaseSample, SpiralGeometry};
use discscape::fitter::fit_rev_p;

#[derive(Args)]
pub struct ProfileFitArgs {
    /// Path to the input disc profile document
    profile: String,
    /// Path to the output profile. Use '-' for output to stdout
    output: String,
    /// Path to measurement file
    #[arg(short = 'm', long = "measurements", value_name = "FILE")]
    measure: String,

    /// Override the ref_radius specified in measurements
    #[arg(long = "ref-radius")]
    ref_radius: Option<f64>,

    /// Write the residual error as correction into output profile
    #[arg(long = "bake")]
    bake: bool,

    /// When --bake is used on a profile with existing correction, overwrite it completely
    #[arg(long = "replace-correction", conflicts_with = "accumulate_correction")]
    replace_correction: bool,

    /// When --bake is used on a profile with existing correction, accumulate residual onto it
    #[arg(long = "accumulate-correction", conflicts_with = "replace_correction")]
    accumulate_correction: bool,
}

/*
    Bake Disabled + Profile without Correction -> Trivial
    Bake Disabled + Profile with Correction    -> Give warning

    Bake Enabled  + Profile without Correction -> Trivial
    Bake Enabled  + Profile with Correction:
        - Reject request
        - Accumulate correction

*/

pub fn profile_fit_command(args: ProfileFitArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("Failed to read profile file at '{}'", args.profile))?;
    let mut profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.profile))?;

    let measure_content = std::fs::read_to_string(&args.measure)
        .with_context(|| format!("Failed to read measure file at '{}'", args.measure))?;
    let measure: MeasurementDoc = serde_json::from_str(&measure_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.profile))?;

    let has_existing_correction = profile.correction.is_some();
    if has_existing_correction && args.bake && !args.replace_correction && !args.accumulate_correction {
        anyhow::bail!(
            "Input profile already contains a correction table.\n\
             Refusing to bake new residuals without an explicit merge strategy.\n\
             Use '--replace-correction' to discard existing correction, or\n\
             Use '--accumulate-correction' to accumulate new residuals on top of it."
        )
    }

    let geometry = SpiralGeometry::try_from(profile.geometry)?;
    let z_radius = args.ref_radius.or(measure.ref_radius).unwrap_or(geometry.r0_mm());
    let error = measure.to_phase_samples();
    let (new_geometry, residual) = fit_rev_p(geometry, &error, z_radius)?;

    let delta_n = new_geometry.revolutions() - geometry.revolutions();
    let delta_p = new_geometry.pitch_um() - geometry.pitch_um();
    let delta_rend = new_geometry.outer_radius_mm() - geometry.outer_radius_mm();

    eprintln!("Geometry");
    eprintln!("  Track Pitch:        {:.9} µm  ({:+.9} µm)"  , new_geometry.pitch_um(), delta_p);
    eprintln!("  Revolutions:        {:.6} rev ({:+.6} rev)" , new_geometry.revolutions(), delta_n);
    eprintln!("  End Radius:         {:.3} mm  ({:+.3} mm)"  , new_geometry.outer_radius_mm(), delta_rend);
    eprintln!();

    let max_abs: f64 = residual.iter()
        .map(|s| s.rev.abs())
        .fold(f64::NEG_INFINITY, f64::max);
    let sum_sq: f64 = residual.iter()
        .map(|s| s.rev * s.rev)
        .sum();
    let rms = (sum_sq / residual.len() as f64).sqrt();

    eprintln!("Residual");
    eprintln!("  Max Residual:      {:+.6} rev", max_abs);
    eprintln!("  Residual RMS:      {:.6} rev", rms);

    profile.geometry = GeometryProfile::from(new_geometry);
    if args.bake {
        if let Some(correction) = &profile.correction && args.accumulate_correction {
            let old_correction = PhaseCorrection::try_from(correction.clone())?;
            let mut samples: Vec<PhaseSample> = vec![];

            for res_sample in &residual {
                let r = res_sample.radius_mm;
                let old_c = old_correction.correction_rev(r);
                let new_c = old_c + res_sample.rev;
                samples.push(PhaseSample { radius_mm: r, rev: new_c });
            }
            let interpo = PhaseCorrectionInterpolation::new(
                samples,
                InterpolationMethod::Linear,
                ExtrapolationMethod::Linear
            );
            profile.correction = Some(CorrectionProfile::Interpolation(interpo.into()));
        } else {
            let interpo = PhaseCorrectionInterpolation::new(
                residual,
                InterpolationMethod::Linear,
                ExtrapolationMethod::Linear
            );
            profile.correction = Some(CorrectionProfile::Interpolation(interpo.into()));
        }
    } else {
        if has_existing_correction {
            eprintln!("[WARN] Input profile already has a correction table. Existing correction was retained.")
        }
    }

    let output_content = serde_json::to_string_pretty(&profile)?;
    if args.output == "-" {
        println!("{output_content}")
    } else {
        let mut file = File::create_new(&args.output)
            .with_context(|| format!("Failed to create or open output file '{}'", args.output))?;
        file.write_all(output_content.as_bytes())
            .with_context(|| format!("Failed to write to output profile '{}'", args.output))?;
    }

    Ok(())
}