use core::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::str::FromStr;

use anyhow::Context;
use clap::{Args, ValueEnum};
use discscape::disc::pattern::Pattern;
use discscape::disc::{Disc, cd, dvd};
use discscape::dither::Dither;
use discscape::profile::DiscProfile;
use discscape::picture::DiscPicture;
use discscape::random::Random;
use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CdFormat {
    Audio,
    Mode1_2352,
    Mode1_2048,
    Mode2_2336,
}

/*
    PatternArg
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum PatternArg {
    /// Short-period modulation pattern
    #[value(name = "short")]
    ShortPeriod,
    /// Long-period modulation pattern
    #[value(name = "long")]
    LongPeriod,
    /// Broadband / aperiodic scatter pattern, configured via --random
    #[value(name = "random", alias = "broadband", alias = "diffuse")]
    Random,
}

impl From<PatternArg> for Pattern {
    fn from(arg: PatternArg) -> Self {
        match arg {
            PatternArg::ShortPeriod => Pattern::ShortPeriod,
            PatternArg::LongPeriod => Pattern::LongPeriod,
            PatternArg::Random => Pattern::Random,
        }
    }
}

/*
    DitherArg
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherArg {
    Gate { threshold: u8 },
    SplitMix64 { seed: u64 }
}

impl DitherArg {
    pub const DEFAULT_THRESHOLD: u8 = 128;
    pub const DEFAULT_SEED: u64 = 0x114514;
}

impl Default for DitherArg {
    fn default() -> Self {
        Self::SplitMix64 {
            seed: Self::DEFAULT_SEED,
        }
    }
}

impl fmt::Display for DitherArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gate { threshold } => write!(f, "gate:{threshold}"),
            Self::SplitMix64 { seed } => write!(f, "splitmix64:0x{seed:x}"),
        }
    }
}

impl FromStr for DitherArg {
    type Err = ParseDitherArgError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, param) = match s.split_once(':') {
            Some((k, p)) => (k.trim(), Some(p.trim())),
            None => (s.trim() , None)
        };

        match kind.to_ascii_lowercase().as_str() {
            "gate" => {
                let threshold = match param {
                    Some(p) => p.parse::<u8>().map_err(|source| 
                        ParseDitherArgError::InvalidThreshold {
                            value: p.to_string(),
                            source
                        }
                    )?,
                    None => Self::DEFAULT_THRESHOLD
                };
                Ok(Self::Gate { threshold })
            }
            "splitmix64" => {
                let seed = match param {
                    Some(p) => {
                        if p.starts_with("0x") {
                            u64::from_str_radix(p.trim_start_matches("0x"), 16).map_err(|source| {
                                ParseDitherArgError::InvalidSeed { value: p.to_string(), source }
                            })?
                        } else {
                            u64::from_str_radix(p, 10).map_err(|source| {
                                ParseDitherArgError::InvalidSeed { value: p.to_string(), source }
                            })?
                        }
                    }
                    None => Self::DEFAULT_SEED,
                };
                Ok(Self::SplitMix64 { seed })
            }
            _ => Err(ParseDitherArgError::UnknownKind { kind: kind.to_string() })
        }
    }
}

impl From<DitherArg> for Dither {
    fn from(value: DitherArg) -> Self {
        match value {
            DitherArg::Gate { threshold } => Self::gate(threshold),
            DitherArg::SplitMix64 { seed } => Self::splitmix64(seed),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseDitherArgError {
    #[error("Unknown dither algorithm '{kind}'. Expected 'gate[:0..=255]' or 'splitmix64[:seed]'")]
    UnknownKind { kind: String },

    #[error("Invalid gate threshold '{value}': expected an integer in 0..=255 ({source})")]
    InvalidThreshold {
        value: String,
        #[source]
        source: std::num::ParseIntError
    },

    #[error("Invalid splitmix64 seed '{value}': expected decimal or 0x-prefixed hex ({source})")]
    InvalidSeed {
        value: String,
        #[source]
        source: std::num::ParseIntError
    }
}

/*
    Random Arg
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomArg {
    Scramble,
    SplitMix64 { seed: u64 }
}

impl RandomArg {
    pub const DEFAULT_SEED: u64 = 0x1919810;
}

impl Default for RandomArg {
    fn default() -> Self {
        Self::SplitMix64 {
            seed: Self::DEFAULT_SEED,
        }
    }
}

impl fmt::Display for RandomArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scramble => write!(f, "scramble"),
            Self::SplitMix64 { seed } => write!(f, "splitmix64:0x{seed:x}"),
        }
    }
}

impl FromStr for RandomArg {
    type Err = ParseRandomArgError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, param) = match s.split_once(':') {
            Some((k, p)) => (k.trim(), Some(p.trim())),
            None => (s.trim() , None)
        };

        match kind.to_ascii_lowercase().as_str() {
            "scramble" | "zero" => {
                if let Some(p) = param {
                    return Err(ParseRandomArgError::UnexpectedParameter {
                        kind: kind.to_string(),
                        param: p.to_string(),
                    });
                }
                Ok(Self::Scramble)
            }
            "splitmix64" => {
                let seed = match param {
                    Some(p) => {
                        if p.starts_with("0x") {
                            u64::from_str_radix(p.trim_start_matches("0x"), 16).map_err(|source| {
                                ParseRandomArgError::InvalidSeed { value: p.to_string(), source }
                            })?
                        } else {
                            u64::from_str_radix(p, 10).map_err(|source| {
                                ParseRandomArgError::InvalidSeed { value: p.to_string(), source }
                            })?
                        }
                    }
                    None => Self::DEFAULT_SEED,
                };
                Ok(Self::SplitMix64 { seed })
            }
            _ => Err(ParseRandomArgError::UnknownKind { kind: kind.to_string() })
        }
    }
}

impl From<RandomArg> for Random {
    fn from(value: RandomArg) -> Self {
        match value {
            RandomArg::Scramble => Random::scramble(),
            RandomArg::SplitMix64 { seed } => Random::splitmix64(seed),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseRandomArgError {
    #[error("Unknown dither algorithm '{kind}'. Expected 'scramble' or 'splitmix64[:seed]'")]
    UnknownKind { kind: String },

    #[error("Random mode '{kind}' does not accept a parameter (got '{param}')")]
    UnexpectedParameter {
        kind: String,
        param: String,
    },

    #[error("Invalid splitmix64 seed '{value}': expected decimal or 0x-prefixed hex ({source})")]
    InvalidSeed {
        value: String,
        #[source]
        source: std::num::ParseIntError
    }
}

/*
    Render command args
*/

#[derive(Debug, Args)]
pub struct RenderArgs {
    /// Path to the input image to be rendered onto the disc
    pub image: std::path::PathBuf,

    /// Path to the output disc image file. Use '-' for output to stdout
    pub output: String,

    /// Path to the disc profile document
    #[arg(long, short = 'p', value_name = "FILE")]
    pub profile: std::path::PathBuf,

    /// Output sector data format
    #[arg(long)]
    pub format: Option<CdFormat>,

    /// Starting logical block address (LBA) on the disc (inclusive)
    #[arg(long, value_name = "N")]
    pub start_sector: Option<u64>,
    
    /// Ending logical block address (LBA) on the disc (exclusive)
    #[arg(long, value_name = "N")]
    pub end_sector: Option<u64>,

    /// Optical pattern for light pixels
    #[arg(
        long,
        value_enum,
        default_value_t = PatternArg::ShortPeriod,
        help_heading = "Rendering Options"
    )]
    pub light: PatternArg,

    /// Optical pattern for dark pixels
    #[arg(
        long,
        value_enum,
        default_value_t = PatternArg::LongPeriod,
        help_heading = "Rendering Options"
    )]
    pub dark: PatternArg,

    /// Dithering algorithm: 'gate[:0..=255]' or 'splitmix64[:seed]'
    #[arg(
        long,
        default_value_t,
        value_name = "SPEC",
        help_heading = "Rendering Options"
    )]
    pub dither: DitherArg,

    /// Broadband generator for 'random' pattern: 'scramble' or 'splitmix64[:seed]'
    #[arg(
        long,
        default_value_t,
        value_name = "SPEC",
        help_heading = "Rendering Options"
    )]
    pub random: RandomArg,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Invalid sector range: start_sector ({start}) must be strictly less than end_sector ({end})")]
    InvalidSectorRange { start: u64, end: u64 },
    #[error("Sector range out of bounds: end_sector ({end}) exceeds disc capacity ({capacity})")]
    SectorOutOfBounds { end: u64, capacity: u64 },
    #[error("CD disc profile requires an output format. Please specify one using '--format <audio|mode1-2352|mode1-2048|mode2-2336>'")]
    MissingCdFormat,
}

pub fn render_command(args: RenderArgs) -> anyhow::Result<()> {
    let profile_content = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("Failed to read profile file at '{}'", args.profile.display()))?;
    let profile: DiscProfile = serde_json::from_str(&profile_content)
        .with_context(|| format!("Failed to parse JSON in profile '{}'", args.profile.display()))?;

    let disc = Disc::try_from(profile)?.with_lut_correction(0.1);
    let capacity = disc.sectors();

    /* Sector size assertion */
    let start_sector = args.start_sector.unwrap_or(0);
    let end_sector = args.end_sector.unwrap_or(capacity);
    if start_sector >= end_sector {
        return Err(RenderError::InvalidSectorRange { start: start_sector, end: end_sector }.into())
    }
    if end_sector > capacity {
        return Err(RenderError::SectorOutOfBounds { end: end_sector, capacity }.into())
    }
    
    let dither = Dither::from(args.dither);
    let random = Random::from(args.random);
    let light = Pattern::from(args.light);
    let dark = Pattern::from(args.dark);
    let use_random = light == Pattern::Random || dark == Pattern::Random;

    if light == dark {
        let name = args.light
            .to_possible_value()
            .map(|v| v.get_name().to_owned())
            .unwrap_or_else(|| format!("{:?}", args.light));
        eprintln!("[WARN] '--light' and '--dark' are both set to '{name}'; the rendered disc will have no image contrast.");
    }

    let image = ImageReader::open(&args.image)
        .with_context(|| format!("Failed to open image file '{}'", args.image.display()))?
        .decode()
        .with_context(|| format!("Failed to decode image file'{}'", args.image.display()))?;
    let picture = DiscPicture::new(image.grayscale());

    let inner: Box<dyn Write> = if &args.output == "-" {
        Box::new(std::io::stdout().lock())
    } else {
        let file = File::create(&args.output)
            .with_context(|| format!("Failed to create or open output file '{}'", args.output))?;
        Box::new(file)
    };
    let mut writer = BufWriter::with_capacity(1024 * 1024, inner);

    match disc {
        Disc::Cd(disc) => {
            let format = args.format.ok_or(RenderError::MissingCdFormat)?;

            if format == CdFormat::Audio && use_random && matches!(random, Random::Scramble) {
                eprintln!("[WARN] CD Audio has no data scrambler; '--random scramble' will emit constant 0x00 bytes (periodic EFM symbols, not broadband scatter). Consider '--random splitmix64' instead.");
            }

            match format {
                CdFormat::Audio => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_audio_chunk(&disc, &picture, &dither, &random, light, dark, i)
                    )?;
                }
                CdFormat::Mode1_2352 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_mode1_2352_chunk(&disc, &picture, &dither, &random, light, dark, i)
                    )?;
                }
                CdFormat::Mode1_2048 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_mode1_2048_chunk(&disc, &picture, &dither, &random, light, dark, i)
                    )?;
                }
                CdFormat::Mode2_2336 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i|
                        cd::generate_mode2_2336_chunk(&disc, &picture, &dither, &random, light, dark, i)
                    )?;
                }
            }
        },
        Disc::Dvd(disc) => {
            if let Some(fmt) = args.format {
                eprintln!("[WARN] '--format {fmt:?}' is ignored because the loaded profile is for a DVD.");
            }

            render_sectors(&mut writer, start_sector, end_sector, |i|
                dvd::generate_sector(&disc, &picture, &dither, &random, light, dark, i)
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