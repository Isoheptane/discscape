
use anyhow::Context;
use clap::{Subcommand, Args};
use discscape::profile::{CorrectionProfile, DiscProfile};
use discscape::geometry::PhaseCorrection;
use discscape::disc::Disc;

#[derive(Subcommand)]
pub enum ProfileCommands {
    Show(ProfileShowArgs),
    Validate(ProfileValidateArgs)
}

pub fn profile_command(cmd: ProfileCommands) -> anyhow::Result<()> {
    match cmd {
        ProfileCommands::Show(args) => {
            profile_show_command(args)
        }
        ProfileCommands::Validate(args) => {
            profile_validate_command(args)
        },
    }
}

#[derive(Args)]
pub struct ProfileShowArgs {
    file: String
}

pub fn profile_show_command(args: ProfileShowArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read profile file at '{}'", args.file))?;
    let profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.file))?;

    let disc = Disc::try_from(profile.clone())?;
    let capacity_sectors = disc.sectors();
    let geometry = disc.geometry();
    let base_geometry = &geometry.geometry;
    let r0_mm = base_geometry.r0_mm();
    let rend_mm = base_geometry.outer_radius_mm();
    let pitch_um = base_geometry.pitch_um();
    let revolutions = base_geometry.revolutions();

    // println!("{:#?}", profile);
    
    println!("Disc Profile: {}\n", args.file);
    println!("Disc Information");
    println!("  Type:               {}", profile.disc.disc_type);
    println!("  Capacity:           {capacity_sectors} sectors");
    println!();
    println!("Geometry");
    println!("  Start Radius:       {r0_mm:.3} mm");
    println!("  End Radius:         {rend_mm:.3} mm");
    println!("  Track Pitch:        {pitch_um:.6} µm");
    println!("  Revolutions:        {revolutions:.6} rev");
    println!();

    match &geometry.phase_correction {
        None => { println!("Correction: None") },
        Some(PhaseCorrection::Interpolation(interp)) => {
            println!("Correction");
            println!("  Type:               Interpolation");
            println!("  Method:             {}", interp.interpolation);
            println!("  Extrapolation:      {}", interp.extrapolation);
            println!("  Sample Count:       {}", interp.samples.len());

            if let (Some(first), Some(last))
            = (interp.samples.first(), interp.samples.last())  {
                println!("    Radius Span:      {:.3} mm ~ {:.3} mm", first.radius_mm, last.radius_mm);
            }

            let min = interp.samples.iter()
                .map(|p| p.correction_rev)
                .fold(f64::INFINITY, f64::min);

            let max = interp.samples.iter()
                .map(|p| p.correction_rev)
                .fold(f64::NEG_INFINITY, f64::max);

            println!("    Value Range:      {:+.6} rev ~ {:+.6} rev", min, max);
        },
        Some(PhaseCorrection::Lut(lut)) => {
            let r0_mm = lut.radius_min;
            let rend_mm = lut.radius_min + lut.step * (lut.lut.len() - 1) as f64;

            println!("Correction");
            println!("  Type:               LUT");
            println!("  LUT Size:           {}", lut.lut.len());
            println!("  Radius Span:        {:.3} mm ~ {:.3} mm", r0_mm, rend_mm);
            println!("  Step:               {:.3} mm", lut.step);

            let min = lut.lut.iter()
                .map(|p| *p)
                .fold(f64::INFINITY, f64::min);

            let max = lut.lut.iter()
                .map(|p| *p)
                .fold(f64::NEG_INFINITY, f64::max);

            println!("  Value Range:        {:+.6} rev ~ {:+.6} rev", min, max);
        }
    }
    println!();

    if let Some(metadata) = profile.metadata {
        if let Some(media) = metadata.media {
            println!("Media Metadata");
            if let Some(v) = media.brand {
                println!("  Brand:              {v}")
            }
            if let Some(v) = media.description {
                println!("  Description:        {v}")
            }
            if let Some(v) = media.mid {
                println!("  Media ID:           {v}")
            }
            if let Some(v) = media.rated_speed {
                println!("  Rated Speed:        {v}")
            }
            println!();
        }
        if let Some(recording) = metadata.recording{
            println!("Recording Metadata");
            if let Some(v) = recording.drive {
                println!("  Drive:              {v}")
            }
            if let Some(v) = recording.firmware {
                println!("  Recorder Firmware:  {v}")
            }
            if let Some(v) = recording.speed_x {
                println!("  Recording Speed:    {v:.1}x")
            }
            if let Some(v) = recording.software {
                println!("  Recording Software: {v}")
            }
            println!();
        }
    }

    Ok(())
}

#[derive(Args)]
pub struct ProfileValidateArgs {
    file: String
}

pub fn profile_validate_command(args: ProfileValidateArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read profile file at '{}'", args.file))?;
    let profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.file))?;

    // let disc = profile.disc;

    let geometry = profile.geometry;
    let r0_mm = geometry.r0_mm;
    let pitch_um = geometry.pitch_um;
    let revolutions = geometry.n;
    let rend_mm = r0_mm + (revolutions * pitch_um) / 1000.0;

    if !r0_mm.is_finite() || r0_mm  < 0.0 {
        println!("[ERRO] r0_mm ({r0_mm}) is not finite or not a positive number")
    }
    if !pitch_um.is_finite() || pitch_um < 0.0 {
        println!("[ERRO] pitch_um ({pitch_um}) is not finite or not a positive number")
    }
    if !revolutions.is_finite() || revolutions < 0.0 {
        println!("[ERRO] n ({revolutions}) is not finite or not a positive number")
    }
    if rend_mm > 60.0 {
        println!("[WARN] Outer radius ({rend_mm}) is beyond 60.0 mm")
    }

    if let Some(correction) = profile.correction {
        match correction {
            CorrectionProfile::Interpolation(interpo) => {
                for (idx, [radius_mm, correction_rev]) in interpo.samples.iter().enumerate() {
                    if !radius_mm.is_finite() || *radius_mm  < 0.0 {
                        println!("[ERRO] sample[{idx}] radius ({radius_mm}) is not finite or not a positive number")
                    }
                    if !correction_rev.is_finite() {
                        println!("[ERRO] sample[{idx}] correction ({correction_rev}) is not finite")
                    }
                }
                for (idx, window) in interpo.samples.windows(2).enumerate() {
                    let ra = window[0][0];
                    let rb = window[1][0];
                    if ra >= rb {
                        println!("[ERRO] sample list radius is not strictly increasing at sample[{idx}]");
                    }
                }
            }
        }
    }

    Ok(())
}