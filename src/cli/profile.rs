use std::{fs::File, io::BufReader};

use clap::{Subcommand, Args};
use discscape::{geometry, profile::{CorrectionProfile, DiscProfile}};

#[derive(Subcommand)]
pub enum ProfileCommands {
    Show(ProfileShowArgs),
    Validate
}

pub fn profile_command(cmd: ProfileCommands) {
    match cmd {
        ProfileCommands::Show(profile_show_args) => {
            profile_show_command(profile_show_args);
        }
        ProfileCommands::Validate => todo!(),
    }
}

#[derive(Args)]
pub struct ProfileShowArgs {
    file: String
}

pub fn profile_show_command(cmd: ProfileShowArgs) {
    let content = std::fs::read_to_string(&cmd.file).expect("Can not read file");
    let profile: DiscProfile = serde_json::from_str(&content).expect("Can not parse string");

    // println!("{:#?}", profile);

    let disc = profile.disc;
    let capacity_sectors = disc.capacity_sectors;

    let geometry = profile.geometry;
    let r0_mm = geometry.r0_mm;
    let pitch_um = geometry.pitch_um;
    let revolutions = geometry.n;
    let rend_mm = r0_mm + (revolutions * pitch_um) / 1000.0;

    let correction = profile.correction;
    
    println!("Disc Profile: {}\n", cmd.file);
    println!("Disc Information");
    println!("  Family:         {} ({})", disc.family, disc.disc_type);
    println!("  Capacity:       {capacity_sectors} sectors");
    println!();
    println!("Geometry");
    println!("  Start Radius:   {r0_mm:.3} mm");
    println!("  End Radius:     {rend_mm:.3} mm");
    println!("  Track Pitch:    {pitch_um:.6} µm");
    println!("  Revolutions:    {revolutions:.6} rev");
    println!();

    match correction {
        None => { println!("Correction: None") },
        Some(CorrectionProfile::Interpolation(interp)) => {
            println!("Correction");
            println!("  Type:           Interpolation");
            println!("  Method:         {}", interp.method);
            println!("  Extrapolation:  {}", interp.extrapolation);
            println!("  Sample Count:   {}", interp.samples.len());

            if let (Some(first), Some(last))
            = (interp.samples.first(), interp.samples.last())  {
                println!("    Radius Span:  {:.2} mm ~ {:.2} mm", first[0], last[0]);
            }

            let min = interp.samples.iter()
                .map(|p| p[1])
                .fold(f64::INFINITY, f64::min);

            let max = interp.samples.iter()
                .map(|p| p[1])
                .fold(f64::NEG_INFINITY, f64::max);

            println!("    Value Range:  {:.6} rev ~ {:.6} rev", min, max);
        }
    }

}