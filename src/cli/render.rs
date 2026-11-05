use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};

use anyhow::Context;
use clap::Args;
use discscape::cd::{self, CdDisc};
use discscape::dither::Dither;
use discscape::dvd::{self, DvdDisc};
use discscape::profile::{CorrectionProfile, DiscFamily, DiscProfile, DiscType};
use discscape::picture::DiscPicture;
use discscape::geometry::{CalibratedSpiralGeometry, PhaseCorrection, PhaseCorrectionInterpolation, PhaseSample, SpiralGeometry};
use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CdFormat {
    Audio,
    Mode1_2352,
    Mode1_2048,
    Mode2_2336,
}

#[derive(Debug, Args)]
pub struct RenderArgs {
    /// Path to the input image to be rendered onto the disc
    pub image: std::path::PathBuf,
    #[arg(long, short = 'p', value_name = "FILE")]

    /// Path to the disc profile document
    pub profile: std::path::PathBuf,
    #[arg(long, short = 'o', value_name = "FILE")]

    /// Path to the output disc image file. Use '-' for output to stdout
    pub output: String,

    /// Output sector data format
    #[arg(long)]
    pub format: Option<CdFormat>,

    /// Starting logical block address (LBA) on the disc (inclusive)
    #[arg(long, value_name = "N")]
    pub start_sector: Option<u64>,
    
    /// Ending logical block address (LBA) on the disc (exclusive)
    #[arg(long, value_name = "N")]
    pub end_sector: Option<u64>,

    /// Seed for picture dither hash
    #[arg(long)]
    pub seed: Option<u64>,

    /// Invert image brightness
    #[arg(long, short = 'i')]
    pub invert: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Invalid sector range: start_sector ({start}) must be strictly less than end_sector ({end})")]
    InvalidSectorRange { start: u64, end: u64 },
    #[error("Sector range out of bounds: end_sector ({end}) exceeds disc capacity ({capacity})")]
    SectorOutOfBounds { end: u64, capacity: u64 },
    #[error("CD disc profile requires an output format. Please specify one using '--format <audio|mode1-2352|mode1-2048|mode2-2336>'")]
    MissingCdFormat,
    #[error("Incompatible profile: DVD cannot have disc type {0:?}")]
    InvalidDvdDiscType(DiscType),
}

pub fn render_command(args: RenderArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("Failed to read profile file at '{}'", args.profile.display()))?;
    let profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.profile.display()))?;

    let disc = profile.disc;

    let start_sector = args.start_sector.unwrap_or(0);
    let end_sector = args.end_sector.unwrap_or(disc.capacity_sectors);
    if start_sector >= end_sector {
        return Err(RenderError::InvalidSectorRange { start: start_sector, end: end_sector }.into())
    }
    if end_sector > disc.capacity_sectors {
        return Err(RenderError::SectorOutOfBounds { end: end_sector, capacity: disc.capacity_sectors }.into())
    }

    let geometry = SpiralGeometry::new(
        profile.geometry.r0_mm,
        profile.geometry.pitch_um,
        profile.geometry.n
    );

    let correction = profile.correction.map(|correction| {
        let correction = match correction {
            CorrectionProfile::Interpolation(interpo) => {
                let samples: Vec<PhaseSample> = interpo.samples.into_iter().map(
                    |[radius_mm, correction_rev]|
                    PhaseSample { radius_mm, correction_rev }
                ).collect();
                let interpo = PhaseCorrectionInterpolation {
                    samples,
                    interpolation: interpo.method
                };
                PhaseCorrection::Samples(interpo)
            }
        };
        // Might better let user control this
        let lut = correction.to_lut(
            geometry.r0_mm(), 
            geometry.outer_radius_mm(), 
            0.1
        );
        PhaseCorrection::Lut(lut)
    });

    let geometry = CalibratedSpiralGeometry {
        geometry,
        phase_correction: correction,
    };

    let inner: Box<dyn Write> = if &args.output == "-" {
        Box::new(std::io::stdout().lock())
    } else {
        let file = File::create(&args.output)
            .with_context(|| format!("Failed to create or open output file '{}'", args.output))?;
        Box::new(file)
    };
    let mut writer = BufWriter::with_capacity(1024 * 1024, inner);

    let image = ImageReader::open(args.image)
        .with_context(|| format!("Failed to open image file '{}'", args.output))?
        .decode()
        .with_context(|| format!("Failed to decode image file'{}'", args.output))?;
    let mut grayscale = image.grayscale();
    if args.invert {
        grayscale.invert();
    }
    let picture = DiscPicture::new(grayscale);

    let dither = Dither::splitmix64(args.seed.unwrap_or(0x114514));

    match disc.family {
        DiscFamily::Cd => {
            let format = args.format.ok_or(RenderError::MissingCdFormat)?;
            let disc = CdDisc::new(geometry, disc.capacity_sectors);
            match format {
                CdFormat::Audio => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_audio_chunk(&disc, &picture, &dither, i)
                    )?;
                }
                CdFormat::Mode1_2352 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_mode1_2352_chunk(&disc, &picture, &dither, i)
                    )?;
                }
                CdFormat::Mode1_2048 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_mode1_2048_chunk(&disc, &picture, &dither, i)
                    )?;
                }
                CdFormat::Mode2_2336 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_mode2_2336_chunk(&disc, &picture, &dither, i)
                    )?;
                }
            }
        },
        DiscFamily::Dvd => {
            if let Some(fmt) = args.format {
                eprintln!("[WARN] '--format {fmt:?}' is ignored because the loaded profile is for a DVD.");
            }

            let disc_type = match disc.disc_type {
                discscape::profile::DiscType::DvdMinusR => discscape::dvd::DvdDiscType::MinusR,
                discscape::profile::DiscType::DvdPlusR => discscape::dvd::DvdDiscType::PlusR,
                discscape::profile::DiscType::DvdMinusRw => discscape::dvd::DvdDiscType::MinusRw,
                discscape::profile::DiscType::DvdPlusRw => discscape::dvd::DvdDiscType::PlusRw,
                discscape::profile::DiscType::CdR |
                discscape::profile::DiscType::CdRw => return Err(RenderError::InvalidDvdDiscType(disc.disc_type).into())
            };
            let disc = DvdDisc::new(geometry, disc_type, disc.capacity_sectors);

            render_sectors(&mut writer, start_sector, end_sector, |i|
                dvd::generate_sector(&disc, &picture, &dither, i)
            )?;
        },
    }

    Ok(())
}

fn render_sectors<W, F, const N: usize>(
    writer: &mut W,
    start_sector: u64,
    end_sector: u64,
    mut generate: F,
) -> anyhow::Result<()>
where
    W: Write,
    F: FnMut(usize) -> [u8; N],
{
    let bar = ProgressBar::new((end_sector - start_sector) * N as u64);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}, ETA: {eta:>3}")
        .expect("Invalid progress bar template")
        .progress_chars("#>.")
    );

    for sector in start_sector..end_sector {
        let chunk = generate(sector as usize);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk)
            .context("Failed to write sector data to output stream")?;
        bar.inc(N as u64);
    }
    
    writer.flush()
        .context("Failed to flush buffer to output data")?;
    bar.finish();

    Ok(())
}