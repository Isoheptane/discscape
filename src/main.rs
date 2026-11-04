mod sin_table;
mod picture;
mod geometry;
mod cd;
mod dvd;
mod efmplus;
mod fitter;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::SmallRng;

use crate::dvd::{DvdDisc, DvdDiscType};
use crate::geometry::{PhaseCorrectionSamples, PhaseSample, SampleInterpolation};
use crate::geometry::{CalibratedSpiralGeometry, PhaseCorrection, SpiralGeometry};
use crate::cd::CdDisc;
use crate::picture::{DiscPicture};

fn main() {
    let start = Instant::now();

    // let geometry = SpiralGeometry::new(
    //     24.830758376375062, 
    //     1.5001855170532903, 
    //     22110.093216189
    // );

    let geometry = SpiralGeometry::new(
        24.0,
        0.7414367253530723,
        45981.86501395135,
    );
    println!("{:#?}", geometry);

    let samples = vec![
        PhaseSample { radius_mm: 25.0, correction_deg: -11.1 },
        PhaseSample { radius_mm: 30.0, correction_deg: -51.8 },
        PhaseSample { radius_mm: 35.0, correction_deg: -49.6 },
        PhaseSample { radius_mm: 40.0, correction_deg: -20.7 },
        PhaseSample { radius_mm: 45.0, correction_deg: 24.0 },
        PhaseSample { radius_mm: 50.0, correction_deg: 87.9 },
        PhaseSample { radius_mm: 55.0, correction_deg: 158.2},
        // PhaseSample { radius_mm: 58.0, correction_deg: 200 },
    ];

    let calibrated_geometry = fitter::fit(&geometry, &samples, 55.0);
    println!("{:#?}", calibrated_geometry);

    let calib_geo = CalibratedSpiralGeometry {
        geometry: calibrated_geometry.unwrap(),
        phase_correction: None
    };

    let disc = DvdDisc::new(calib_geo, DvdDiscType::PlusR, 2_295_104);

    let image = ImageReader::open("spoke.png")
        .expect("Failed to open")
        .decode().expect("Failed to decode");

    let picture = DiscPicture::new(image);

    let file = match File::create("spoke.iso") {
        Ok(f) => f,
        Err(e) => {
            panic!("Can't open file: {}", e)
        }
    };


    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    let start_sector: usize = 0;
    let end_sector: usize = 2_295_104;
    let sector_count: usize = end_sector - start_sector;

    println!("Start Position: {:#?}", disc.geometry().polar_at_pos(0.0));
    println!("End Position: {:#?}", disc.geometry().polar_at_pos(1.0));
    println!("End Radius: {:#?}", disc.geometry().geometry.outer_radius_mm());

    let mut rng: SmallRng = rand::make_rng();

    let bar = ProgressBar::new(sector_count as u64 * 2048);

    // bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len} sectors")
    //     .unwrap()
    //     .progress_chars("##.")
    // );
    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    for sector in start_sector..end_sector {

        let chunk = dvd::generate_sector(&disc, &picture, &mut rng, sector);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        // writer.write_all(&chunk).expect("Failed to write to writer");

        bar.inc(2048);

    }
    // writer.flush().expect("Failed to flush buffer");
    bar.finish();

    let end = Instant::now();

    let ns = end.duration_since(start).as_nanos();
    println!("Job done in {:.3} seconds", ns as f64 / 1_000_000_000.0)

    
}