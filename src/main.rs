mod geometry;
mod cd;

use std::fs::File;
use std::io::{BufWriter, Write};

use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use rand::rngs::SmallRng;

use crate::geometry::{PhaseCorrectionLut, PhaseCorrectionSamples, PhaseSample, SampleInterpolation};
use crate::{cd::CdDisc, geometry::{CalibratedSpiralGeometry, PhaseCorrection, SpiralGeometry}};




fn main() {

    let geometry = SpiralGeometry {
        r0: 25.0,
        p: 1.510410490,
        n: 22110.093216189,
    };

    let samples = vec![
        PhaseSample { radius_mm: 25.0, correction_deg: 0.0 },
        PhaseSample { radius_mm: 30.0, correction_deg: 0.835 },
        PhaseSample { radius_mm: 35.0, correction_deg: 0.373 },
        PhaseSample { radius_mm: 40.0, correction_deg: 0.076 },
        PhaseSample { radius_mm: 45.0, correction_deg: -0.478 },
        PhaseSample { radius_mm: 50.0, correction_deg: -0.403 },
        PhaseSample { radius_mm: 55.0, correction_deg: -0.206 },
        PhaseSample { radius_mm: 58.0, correction_deg: 0.0 },
    ];

    // let linear = PhaseCorrectionSamples::new(samples.clone(), SampleInterpolation::Linear);
    let pchip = PhaseCorrectionSamples::new(samples.clone(), SampleInterpolation::Pchip);

    let calib = PhaseCorrection::Samples(pchip.clone());
    let lut = PhaseCorrection::Lut(calib.to_lut(25.0, 58.0, 0.1));

    // for i in 0..100 {
    //     let p1 = linear.correction_rad(25.0 + 0.1 * i as f64);
    //     let p2 = pchip.correction_rad(25.0 + 0.1 * i as f64);
    //     let p3 = lut.correction_rad(25.0 + 0.1 * i as f64);
    // 
    //     println!("{p1:.4}, {p2:.4}, {p3:.4}");
    //     println!("{:.4}, {:.4}", p2 - p1, p3 - p2);
    // }

    let calib_geometry = CalibratedSpiralGeometry {
        geometry,
        phase_correction: Some(lut)
    };

    let disc = CdDisc {
        geometry: calib_geometry,
        capacity_sectors: 359_849,
    };

    let image = ImageReader::open("spoke3.png")
        .expect("Failed to open")
        .decode().expect("Failed to decode")
        .to_luma8();

    let file = match File::create("spoke3-2336.bin") {
        Ok(f) => f,
        Err(e) => {
            panic!("Can't open file: {}", e)
        }
    };


    let mut writer = BufWriter::with_capacity(2352 * 75 * 100, file);

    let start_sector: usize = 0;
    let end_sector: usize = 359_845;
    let sector_count: usize = end_sector - start_sector;
    let total_size: usize = sector_count * 2352;

    // println!("Start Position: {:#?}", disc.geometry.polar_at_pos(0.0));
    // println!("End Position: {:#?}", disc.geometry.polar_at_pos(1.0));
    // println!("End Radius: {:#?}", disc.geometry.geometry.outer_radius_mm());

    let mut rng: SmallRng = rand::make_rng();

    let bar = ProgressBar::new(sector_count as u64);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len}")
        .unwrap()
    );

    for sector in start_sector..end_sector {

        let chunk = cd::generate_mode2_2336_chunk(&disc, &image, &mut rng, sector);

        writer.write_all(&chunk).expect("Failed to write to writer");

        bar.inc(1);

    }
    writer.flush().expect("Failed to flush buffer");
    bar.finish();

    // file.write_all(&bytes).expect("Failed to write.");
}
