
use std::str::{FromStr};

use anyhow::Context;
use clap::{Args, ValueEnum};
use discscape::sampler::MultiplexSampler;
use discscape::picture::DiscPicture;
use discscape::disc::pattern::Pattern;
use image::ImageReader;
use crate::cli::render::{CommonRenderArgs, PatternArg, common_render_command};


#[derive(Debug, Clone)]
pub struct ChannelArg {
    pub image: std::path::PathBuf,
    pub pattern: PatternArg,
    pub weight: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseChannelArgError {
    #[error("Invalid channel spec '{spec}': expected '<pattern>:<image_path>[:<weight>]'")]
    InvalidFormat { spec: String },

    #[error("Unknown pattern '{pattern}' in channel spec: {message}")]
    UnknownPattern { pattern: String, message: String },

    #[error("Invalid channel weight '{weight}': must be a finite number in 0.0..=1.0")]
    InvalidWeight {
        weight: String,
        #[source]
        source: std::num::ParseFloatError
    },

    #[error("Channel weight must be a finite positive number (got {weight})")]
    NonPositiveWeight { weight: f64 },
}

impl FromStr for ChannelArg {
    type Err = ParseChannelArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s.rsplit_once(':').ok_or_else(|| {
            ParseChannelArgError::InvalidFormat { spec: s.to_string() }
        })?;

        // Rightmost is Pattern
        if let Ok(pattern) = PatternArg::from_str(right.trim(), true) {
            if left.is_empty() {
                return Err(ParseChannelArgError::InvalidFormat { spec: s.to_string() })
            }
            return Ok(Self {
                image: std::path::PathBuf::from(left),
                pattern,
                weight: None,
            })
        }

        // Rightmost is NOT pattern arg, try parse as weight
        let weight_val = right.trim().parse::<f64>().map_err(|source| {
            ParseChannelArgError::InvalidWeight { weight: right.to_string(), source }
        })?;

        if !weight_val.is_finite() || weight_val <= 0.0 {
            return Err(ParseChannelArgError::NonPositiveWeight { weight: weight_val });
        }

        let (path_str, pat_str) = left.rsplit_once(':').ok_or_else(|| {
            ParseChannelArgError::InvalidFormat { spec: s.to_string() }
        })?;

        if path_str.is_empty() {
            return Err(ParseChannelArgError::InvalidFormat { spec: s.to_string() });
        }

        let pattern = PatternArg::from_str(pat_str.trim(), true).map_err(|message| {
            ParseChannelArgError::UnknownPattern {
                pattern: pat_str.trim().to_string(),
                message
            }
        })?;

        Ok(Self {
            image: std::path::PathBuf::from(path_str),
            pattern,
            weight: Some(weight_val),
        })
    }
}

#[derive(Debug, Args)]
pub struct RenderMultiplexArgs {
    /// Image channel specifications: '<path>:<pattern>[:<weight>]'
    #[arg(
        long = "channel",
        short = 'c',
        value_name = "SPEC",
        required = true,
        help_heading = "Multiplexing Options"
    )]
    pub channels: Vec<ChannelArg>,

    #[command(flatten)]
    pub common_args: CommonRenderArgs,

    /// Optical pattern for unassigned energy (dump channel)
    #[arg(
        long,
        value_enum,
        default_value_t = PatternArg::PeriodLong,
        help_heading = "Rendering Options"
    )]
    pub dump: PatternArg,

    #[arg(
        long,
        default_value_t = 0x114514,
        value_name = "SEED",
        help_heading = "Rendering Options",
        value_parser = hex_parser
    )]
    pub dither_seed: u64,
}

fn hex_parser(s: &str) -> Result<u64, String> {
    if s.starts_with("0x") {
        u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|_| {
            format!("Invalid dither seed, expected decimal or 0x-prefixed hex seed (get {s})")
        })
    } else {
        u64::from_str_radix(s, 10).map_err(|_| {
            format!("Invalid dither seed, expected decimal or 0x-prefixed hex seed (get {s})")
        })
    }
}

pub fn render_multiplex_command(args: RenderMultiplexArgs) -> anyhow::Result<()> {
    let common = args.common_args;
    
    let mut pats: Vec<(Pattern, DiscPicture, f64)> = vec![];

    for channel in args.channels {
        let image = ImageReader::open(&channel.image)
            .with_context(|| format!("Failed to open image file '{}'", channel.image.display()))?
            .decode()
            .with_context(|| format!("Failed to decode image file'{}'", channel.image.display()))?;
        let picture = DiscPicture::new(image.grayscale());

        let pattern: Pattern = channel.pattern.into();

        pats.push((pattern, picture, channel.weight.unwrap_or(1.0)));
    }

    // Allow seed later
    let sampler = MultiplexSampler::new(pats, args.dump.into(), args.dither_seed, common.gamma);

    common_render_command(common, sampler)
}