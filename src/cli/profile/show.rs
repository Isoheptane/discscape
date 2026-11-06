use anyhow::Context;
use clap::Args;
use discscape::geometry::correction::PhaseCorrection;
use discscape::profile::DiscProfile;
use discscape::disc::Disc;
use serde_json::Value;

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
                .map(|p| p.rev)
                .fold(f64::INFINITY, f64::min);

            let max = interp.samples.iter()
                .map(|p| p.rev)
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
        if let Some(media) = metadata.get("media") {
            println!("Media Metadata");
            if let Some(Value::String(v)) = media.get("brand") {
                println!("  Brand:              {v}")
            }
            if let Some(Value::String(v)) = media.get("description") {
                println!("  Description:        {v}")
            }
            if let Some(Value::String(v)) = media.get("mid") {
                println!("  Media ID:           {v}")
            }
            if let Some(Value::String(v)) = media.get("rated_speed") {
                println!("  Rated Speed:        {v}")
            }
            println!();
        }

        if let Some(recording) = metadata.get("recording") {
            println!("Recording Metadata");
            if let Some(Value::String(v)) = recording.get("drive") {
                println!("  Drive:              {v}")
            }
            if let Some(Value::String(v)) = recording.get("firmware") {
                println!("  Recorder Firmware:  {v}")
            }
            if let Some(Value::Number(v)) = recording.get("speed_x") {
                println!("  Recording Speed:    {v}x",)
            }
            if let Some(Value::String(v)) = recording.get("software") {
                println!("  Recording Software: {v}")
            }
            println!();
        }

        if let Some(notes) = metadata.get("notes") {
            match notes {
                Value::Array(array) => {
                    println!("Notes");
                    for line in array.iter().filter_map(|v| v.as_str()) {
                        println!("  - {line}");
                    }
                    println!();
                }
                Value::String(s) => {
                    println!("Notes");
                    println!("  {s}");
                    println!();
                }
                _ => {}
            };
        }
    }

    Ok(())
}