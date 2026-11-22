pub mod grayscale;
pub mod multiplex;

use core::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::str::FromStr;
use std::sync::mpsc;

use anyhow::Context;
use clap::Args;
use discscape::disc::pattern::Pattern;
use discscape::disc::{Disc, cd, dvd};
use discscape::dither::Dither;
use discscape::profile::DiscProfile;
use discscape::random::Random;
use discscape::sampler::PatternSampler;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

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
    /// Shortest-period modulation pattern
    #[value(name = "shortest", alias = "3t", alias = "xs", alias = "p0")]
    PeriodShortest,
    #[value(name = "x5s1", alias = "p1", hide = true)]
    MacroX5S1,
    #[value(name = "x4s2", alias = "p2", hide = true)]
    MacroX4S2,
    #[value(name = "x3s3", alias = "p3", hide = true)]
    MacroX3S3,
    #[value(name = "x2s4", alias = "p4", hide = true)]
    MacroX2S4,
    #[value(name = "x1s5", alias = "p5", hide = true)]
    MacroX1S5,

    /// Short-period modulation pattern
    #[value(name = "short", alias = "4t", alias = "s", alias = "p6")]
    PeriodShort,
    #[value(name = "s5m1", alias = "p7", hide = true)]
    MacroS5M1,
    #[value(name = "s4m2", alias = "p8", hide = true)]
    MacroS4M2,
    #[value(name = "s3m3", alias = "p9", hide = true)]
    MacroS3M3,
    #[value(name = "s2m4", alias = "p10", hide = true)]
    MacroS2M4,
    #[value(name = "s1m5", alias = "p11", hide = true)]
    MacroS1M5,

    /// Medium-period modulation pattern
    #[value(name = "medium", alias = "5t", alias = "6t", alias = "m", alias = "p12")]
    PeriodMedium,
    #[value(name = "m5l1", alias = "p13", hide = true)]
    MacroM5L1,
    #[value(name = "m4l2", alias = "p14", hide = true)]
    MacroM4L2,
    #[value(name = "m3l3", alias = "p15", hide = true)]
    MacroM3L3,
    #[value(name = "m2l4", alias = "p16", hide = true)]
    MacroM2L4,
    #[value(name = "m1l5", alias = "p17", hide = true)]
    MacroM1L5,

    /// Long-period modulation pattern
    #[value(name = "long", alias = "8t", alias = "l", alias = "p18")]
    PeriodLong,
    
    /// Broadband / aperiodic scatter pattern, configured via --random
    #[value(name = "random", alias = "broadband", alias = "diffuse")]
    Random,
}

impl From<PatternArg> for Pattern {
    fn from(arg: PatternArg) -> Self {
        match arg {
            PatternArg::PeriodShortest  => Pattern::PeriodShortest,
            PatternArg::MacroX5S1       => Pattern::MacroX5S1,
            PatternArg::MacroX4S2       => Pattern::MacroX4S2,
            PatternArg::MacroX3S3       => Pattern::MacroX3S3,
            PatternArg::MacroX2S4       => Pattern::MacroX2S4,
            PatternArg::MacroX1S5       => Pattern::MacroX1S5,
            PatternArg::PeriodShort     => Pattern::PeriodShort,
            PatternArg::MacroS5M1       => Pattern::MacroS5M1,
            PatternArg::MacroS4M2       => Pattern::MacroS4M2,
            PatternArg::MacroS3M3       => Pattern::MacroS3M3,
            PatternArg::MacroS2M4       => Pattern::MacroS2M4,
            PatternArg::MacroS1M5       => Pattern::MacroS1M5,
            PatternArg::PeriodMedium    => Pattern::PeriodMedium,
            PatternArg::MacroM5L1       => Pattern::MacroM5L1,
            PatternArg::MacroM4L2       => Pattern::MacroM4L2,
            PatternArg::MacroM3L3       => Pattern::MacroM3L3,
            PatternArg::MacroM2L4       => Pattern::MacroM2L4,
            PatternArg::MacroM1L5       => Pattern::MacroM1L5,
            PatternArg::PeriodLong      => Pattern::PeriodLong,
            PatternArg::Random          => Pattern::Random,
        }
    }
}

/*
    DitherArg
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherArg {
    SplitMix64 { seed: u64 }
}

impl DitherArg {
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
            DitherArg::SplitMix64 { seed } => Self::new(seed),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseDitherArgError {
    #[error("Unknown dither algorithm '{kind}'. Expected 'splitmix64[:seed]'")]
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
    #[error("Unknown random algorithm '{kind}'. Expected 'scramble' or 'splitmix64[:seed]'")]
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

#[derive(Debug, Args)]
pub struct CommonRenderArgs {
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

    /// Broadband generator for 'random' pattern: 'scramble' or 'splitmix64[:seed]'
    #[arg(
        long,
        default_value_t,
        value_name = "SPEC",
        help_heading = "Rendering Options"
    )]
    pub random: RandomArg,

    /// Gamma correction exponent applied to input image luminance
    #[arg(
        long,
        value_name = "GAMMA",
        default_value_t = 2.2,
        help_heading = "Rendering Options"
    )]
    pub gamma: f64
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

pub fn common_render_command<S: PatternSampler + Sync>(
    args: CommonRenderArgs,
    sampler: S
) -> anyhow::Result<()> {

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
    let random: Random = args.random.into();

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

            if format == CdFormat::Audio && sampler.use_random() && matches!(random, Random::Scramble) {
                eprintln!("[WARN] CD Audio has no data scrambler; '--random scramble' will emit constant 0x00 bytes (periodic EFM symbols, not broadband scatter). Consider '--random splitmix64' instead.");
            }

            match format {
                CdFormat::Audio => {
                    render_sectors(&mut writer, start_sector, end_sector, |i, out|
                        cd::generate_audio_chunk(&disc, &sampler, &random, i, out)
                    )?;
                }
                CdFormat::Mode1_2352 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i, out|
                        cd::generate_mode1_2352_chunk(&disc, &sampler, &random, i, out)
                    )?;
                }
                CdFormat::Mode1_2048 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i, out|
                        cd::generate_mode1_2048_chunk(&disc, &sampler, &random, i, out)
                    )?;
                }
                CdFormat::Mode2_2336 => {
                    render_sectors(&mut writer, start_sector, end_sector, |i, out|
                        cd::generate_mode2_2336_chunk(&disc, &sampler, &random, i, out)
                    )?;
                }
            }
        },
        Disc::Dvd(disc) => {
            if let Some(fmt) = args.format {
                eprintln!("[WARN] '--format {fmt:?}' is ignored because the loaded profile is for a DVD.");
            }

            render_sectors(&mut writer, start_sector, end_sector, |i, out|
                dvd::generate_sector(&disc, &sampler, &random, i, out)
            )?;
        },
    }

    Ok(())
}

fn render_sectors<W, F, const N: usize>(
    writer: &mut W,
    start_sector: u64,
    end_sector: u64,
    generate: F,
) -> anyhow::Result<()>
where
    W: Write,
    F: Fn(usize, &mut [u8; N]) + Sync,
{
    let total_sectors = end_sector - start_sector;
    let bar = ProgressBar::new(total_sectors * N as u64);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}, ETA: {eta:>3}")
        .expect("Invalid progress bar template")
        .progress_chars("#>.")
    );

    const BATCH_SECTORS: usize = 1024;
    const PIPELINE_DEPTH: usize = 3;

    let io_result = std::thread::scope(|s| -> anyhow::Result<()> {
        
        let (free_tx, free_rx) = mpsc::channel::<Vec<[u8; N]>>();
        let (ready_tx, ready_rx) = mpsc::channel::<(usize, Vec<[u8; N]>)>();

        for _ in 0..PIPELINE_DEPTH {
            free_tx
                .send(vec![[0u8; N]; BATCH_SECTORS])
                .expect("Failed to initialize buffer pool")
        }

        let generate = &generate;
        s.spawn(move || {
            let mut current = start_sector;
            while current < end_sector {
                let Ok(mut buf) = free_rx.recv() else {
                    break;
                };

                let batch_len = ((end_sector - current) as usize).min(BATCH_SECTORS);

                buf[..batch_len]
                    .par_iter_mut()
                    .enumerate()
                    .with_min_len(8)
                    .for_each(|(offset, sector_buf)| {
                        generate(current as usize + offset, sector_buf)
                    });

                if ready_tx.send((batch_len, buf)).is_err() {
                    break;
                }

                current += batch_len as u64;
            }
        });

        let mut write_res = Ok(());
        while let Ok((batch_len, buf)) = ready_rx.recv() {
            if let Err(e) = writer.write_all(buf[..batch_len].as_flattened()) {
                write_res = Err(e).context("Failed to write sector data to output stream");
                break;
            }
            bar.inc((batch_len * N) as u64);
            // Reuse buf, send to free immediately
            let _ = free_tx.send(buf);
        }

        drop(ready_rx);
        drop(free_tx);

        write_res
    });

    io_result?;
    writer.flush().context("Failed to flush buffer to output stream")?;
    bar.finish();

    Ok(())
}