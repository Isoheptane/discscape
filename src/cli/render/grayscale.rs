
use anyhow::Context;
use clap::{Args, ValueEnum};
use discscape::disc::pattern::Pattern;
use discscape::picture::DiscPicture;
use discscape::dither::Dither;
use discscape::sampler::GrayscaleSampler;
use image::ImageReader;

use crate::cli::render::{CommonRenderArgs, DitherArg, PatternArg, common_render_command};

/*
    Render command args
*/

#[derive(Debug, Args)]
pub struct RenderGrayscaleArgs {
    /// Path to the input image to be rendered onto the disc
    pub image: std::path::PathBuf,

    #[command(flatten)]
    pub common_args: CommonRenderArgs,

    /// Optical pattern for light pixels
    #[arg(
        long,
        value_enum,
        default_value_t = PatternArg::PeriodShortest,
        help_heading = "Rendering Options"
    )]
    pub light: PatternArg,

    /// Optical pattern for dark pixels
    #[arg(
        long,
        value_enum,
        default_value_t = PatternArg::PeriodLong,
        help_heading = "Rendering Options"
    )]
    pub dark: PatternArg,

    /// Dithering algorithm: 'splitmix64[:seed]'
    #[arg(
        long,
        default_value_t,
        value_name = "SPEC",
        help_heading = "Rendering Options"
    )]
    pub dither: DitherArg,
}

pub fn render_grayscale_command(args: RenderGrayscaleArgs) -> anyhow::Result<()> {
    let common = args.common_args;

    let dither = Dither::from(args.dither);
    let light = Pattern::from(args.light);
    let dark = Pattern::from(args.dark);

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

    let sampler = GrayscaleSampler::new(picture, dither, light, dark, common.gamma);

    common_render_command(common, sampler)
}