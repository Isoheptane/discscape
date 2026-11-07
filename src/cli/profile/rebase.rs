use std::io::Write;
use std::fs::File;

use anyhow::Context;
use clap::Args;
use discscape::profile::{CorrectionProfile, DiscProfile, GeometryProfile};
use discscape::geometry::{PhaseSample, SpiralGeometry};
use discscape::geometry::correction::{PhaseCorrection, PhaseCorrectionInterpolation};
use discscape::fitter::{FittingError, fit_p_fixed_rev, fit_rev_p};

#[derive(Args)]
pub struct ProfileRebaseArgs {
    /// Path to the input disc profile document
    profile: String,
    /// Path to the output profile. Use '-' for output to stdout
    output: String,

    /// The correction reference radius, default to r0 if not set.
    #[arg(long = "ref-radius")]
    ref_radius: Option<f64>,

    /// Do not constrain phase error at outer radius to be strictly zero
    #[arg(short = 'u', long = "unconstrained")]
    unconstrained: bool,

    #[arg(long = "drop-correction")]
    /// Drop correction in output profile
    drop_correction: bool
}

pub fn profile_rebase_command(args: ProfileRebaseArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("Failed to read profile file at '{}'", args.profile))?;
    let mut profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.profile))?;

    let geometry = SpiralGeometry::try_from(profile.geometry)?;
    let Some(correction) = profile.correction else {
        eprintln!("[ERRO] Input profile does not have correction");
        return Ok(());
    };
    let correction = PhaseCorrection::try_from(correction)?;
    let z_radius = args.ref_radius.unwrap_or(geometry.r0_mm());

    let (new_geometry, new_correction) = match correction {
        PhaseCorrection::Interpolation(interpo) => {
            let (geometry, interpo) = rebase_interpolation_correction(
                geometry, 
                interpo, 
                z_radius, 
                args.unconstrained
            )?;
            (geometry, PhaseCorrection::Interpolation(interpo))
        },
        _ => unimplemented!(),
    };

    let delta_n = new_geometry.revolutions() - geometry.revolutions();
    let delta_p = new_geometry.pitch_um() - geometry.pitch_um();
    let delta_rend = new_geometry.outer_radius_mm() - geometry.outer_radius_mm();

    eprintln!("Geometry");
    eprintln!("  Track Pitch:        {:.9} µm  ({:+.9} µm)"  , geometry.pitch_um(), delta_p);
    eprintln!("  Revolutions:        {:.6} rev ({:+.6} rev)" , geometry.revolutions(), delta_n);
    eprintln!("  End Radius:         {:.3} mm  ({:+.3} mm)"  , geometry.outer_radius_mm(), delta_rend);
    eprintln!();

    if let PhaseCorrection::Interpolation(interpo) = &new_correction {
        let max_abs: f64 = interpo.samples.iter()
            .map(|s| s.rev.abs())
            .fold(f64::NEG_INFINITY, f64::max);
        let sum_sq: f64 = interpo.samples.iter()
            .map(|s| s.rev * s.rev)
            .sum();
        let rms = sum_sq / interpo.samples.len() as f64;

        eprintln!("Correction");
        eprintln!("  Max Correction:        {:+.6} rev", max_abs);
        eprintln!("  Correction RMS:        {:.6} rev", rms);
    }

    profile.geometry = GeometryProfile::from(new_geometry);
    profile.correction = if args.drop_correction {
        None
    } else {
        Some(CorrectionProfile::from(new_correction))
    };

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

fn rebase_interpolation_correction(
    geometry: SpiralGeometry,
    correction: PhaseCorrectionInterpolation,
    z_radius: f64,
    unconstrained: bool,
) -> Result<(SpiralGeometry, PhaseCorrectionInterpolation), FittingError> {
    let correction_z = correction.correction_rev(z_radius);
    let error: Vec<PhaseSample> = correction.samples.iter().map(|s| PhaseSample {
        radius_mm: s.radius_mm,
        rev: s.rev - correction_z,
    }).collect();

    let (new_geometry, residual) = if unconstrained {
        fit_rev_p(geometry, &error, z_radius)?
    } else {
        let outer_radius = geometry.outer_radius_mm();
        let delta_n = correction.correction_rev(outer_radius) - correction_z;
        fit_p_fixed_rev(geometry, &error, z_radius, delta_n)?
    };

    let new_correction = PhaseCorrectionInterpolation::new(
        residual,
        correction.interpolation,
        correction.extrapolation
    );

    Ok((new_geometry, new_correction))
}