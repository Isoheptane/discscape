use std::fs::File;
use std::io::{BufWriter, Write};

use clap::Args;
use discscape::cd::CdDisc;
use discscape::dither::Dither;
use discscape::dvd::DvdDisc;
use discscape::profile::{CorrectionProfile, DiscFamily, DiscProfile};
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
    
    /// Overwrite the destination file if it already exists
    #[arg(long)]
    pub overwrite: bool,
}

pub fn render_command(args: RenderArgs) {
    let profile_content = std::fs::read_to_string(&args.profile).expect("Can not read file");
    let profile: DiscProfile = serde_json::from_str(&profile_content).expect("Can not parse string");

    let start_sector = args.start_sector.unwrap_or(0);
    let end_sector = args.end_sector.unwrap_or(profile.disc.capacity_sectors);

    // Replace this to more friendly error later
    assert!(end_sector > start_sector);
    assert!(end_sector - start_sector <= profile.disc.capacity_sectors);

    if profile.disc.family == DiscFamily::Cd && args.format.is_none() {
        // Replace this to more friendly error later
        panic!("Please specify output format for CD")
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
    } else if 
        !std::fs::exists(&args.output).expect("Failed to check file exist") 
        || args.overwrite {
        let file = File::create(args.output)
            .expect("Can't open file or output");
        Box::new(file)
    } else {
        panic!("File already exists")
    };
    let mut writer = BufWriter::with_capacity(1024 * 1024, inner);

    let image = ImageReader::open(args.image)
        .expect("Failed to open")
        .decode().expect("Failed to decode");
    let mut grayscale = image.grayscale();
    if args.invert {
        grayscale.invert();
    }
    let picture = DiscPicture::new(grayscale);

    let dither = Dither::splitmix64(0x114514);

    match profile.disc.family {
        DiscFamily::Cd => {
            let disc = CdDisc::new(geometry, profile.disc.capacity_sectors);
            match args.format.unwrap() {
                CdFormat::Audio => cd_render_audio(&disc, &picture, &dither, start_sector, end_sector, &mut writer),
                CdFormat::Mode1_2352 => cd_render_mode1_2352(&disc, &picture, &dither, start_sector, end_sector, &mut writer),
                CdFormat::Mode1_2048 => cd_render_mode1_2048(&disc, &picture, &dither, start_sector, end_sector, &mut writer),
                CdFormat::Mode2_2336 => cd_render_mode2_2336(&disc, &picture, &dither, start_sector, end_sector, &mut writer),
            }
        },
        DiscFamily::Dvd => {
            let disc_type = match profile.disc.disc_type {
                discscape::profile::DiscType::CdR => panic!("DVD should not have disc type CD-R"),
                discscape::profile::DiscType::CdRw => panic!("DVD should not have disc type CD-RW"),
                discscape::profile::DiscType::DvdMinusR => discscape::dvd::DvdDiscType::MinusR,
                discscape::profile::DiscType::DvdPlusR => discscape::dvd::DvdDiscType::PlusR,
                discscape::profile::DiscType::DvdMinusRw => discscape::dvd::DvdDiscType::MinusRw,
                discscape::profile::DiscType::DvdPlusRw => discscape::dvd::DvdDiscType::PlusRw,
            };
            let disc = DvdDisc::new(geometry, disc_type, profile.disc.capacity_sectors);

            dvd_render(&disc, &picture, &dither, start_sector, end_sector, &mut writer);
        },
    }

}

fn dvd_render(
    disc: &DvdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    start_sector: u64,
    end_sector: u64,
    writer: &mut BufWriter<Box<dyn Write>>,
) {
    let bar = ProgressBar::new((end_sector - start_sector) * 2048);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    for sector in start_sector..end_sector {
        let chunk = discscape::dvd::generate_sector(disc, picture, dither, sector as usize);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk).expect("Failed to write to writer");
        bar.inc(2048);
    }
    
    writer.flush().expect("Failed to flush buffer");
    bar.finish();
}

fn cd_render_audio(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    start_sector: u64,
    end_sector: u64,
    writer: &mut BufWriter<Box<dyn Write>>,
) {
    let bar = ProgressBar::new((end_sector - start_sector) * 2352);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    for sector in start_sector..end_sector {
        let chunk = discscape::cd::generate_audio_chunk(disc, picture, dither, sector as usize);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk).expect("Failed to write to writer");
        bar.inc(2352);
    }
    
    writer.flush().expect("Failed to flush buffer");
    bar.finish();
}

fn cd_render_mode1_2352(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    start_sector: u64,
    end_sector: u64,
    writer: &mut BufWriter<Box<dyn Write>>,
) {
    let bar = ProgressBar::new((end_sector - start_sector) * 2352);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    for sector in start_sector..end_sector {
        let chunk = discscape::cd::generate_mode1_2352_chunk(disc, picture, dither, sector as usize);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk).expect("Failed to write to writer");
        bar.inc(2352);
    }
    
    writer.flush().expect("Failed to flush buffer");
    bar.finish();
}

fn cd_render_mode1_2048(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    start_sector: u64,
    end_sector: u64,
    writer: &mut BufWriter<Box<dyn Write>>,
) {
    let bar = ProgressBar::new((end_sector - start_sector) * 2048);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    for sector in start_sector..end_sector {
        let chunk = discscape::cd::generate_mode1_2048_chunk(disc, picture, dither, sector as usize);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk).expect("Failed to write to writer");
        bar.inc(2048);
    }
    
    writer.flush().expect("Failed to flush buffer");
    bar.finish();
}

fn cd_render_mode2_2336(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    start_sector: u64,
    end_sector: u64,
    writer: &mut BufWriter<Box<dyn Write>>,
) {
    let bar = ProgressBar::new((end_sector - start_sector) * 2336);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    for sector in start_sector..end_sector {
        let chunk = discscape::cd::generate_mode2_2336_chunk(disc, picture, dither, sector as usize);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk).expect("Failed to write to writer");
        bar.inc(2336);
    }
    
    writer.flush().expect("Failed to flush buffer");
    bar.finish();
}