mod geometry;
mod cd;

use std::fs::File;
use std::io::{BufWriter, Write};

use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use rand::RngExt;
use rand::rngs::SmallRng;

use crate::geometry::PhaseSample;
use crate::{cd::CdDisc, geometry::{CalibratedSpiralGeometry, PhaseCalibration, SpiralGeometry}};




fn main() {

    let geometry = SpiralGeometry {
        r0: 25.0,
        p: 1.510410490,
        n: 22110.093216189,
    };

    let calib_geometry = CalibratedSpiralGeometry {
        geometry,
        calibration: PhaseCalibration { samples: vec![
            PhaseSample { radius_mm: 25.0, correction_deg: 0.0 },
            PhaseSample { radius_mm: 30.0, correction_deg: 0.835 },
            PhaseSample { radius_mm: 35.0, correction_deg: 0.373 },
            PhaseSample { radius_mm: 40.0, correction_deg: 0.076 },
            PhaseSample { radius_mm: 45.0, correction_deg: -0.478 },
            PhaseSample { radius_mm: 50.0, correction_deg: -0.403 },
            PhaseSample { radius_mm: 55.0, correction_deg: -0.206 },
            PhaseSample { radius_mm: 58.0, correction_deg: 0.0 },
        ] }
    };

    let disc = CdDisc {
        geometry: calib_geometry,
        capacity_sectors: 359_849,
    };

    let spoke_image = ImageReader::open("salt_contrast.png")
        .expect("Failed to open")
        .decode().expect("Failed to decode")
        .to_luma8();
    let image_w = spoke_image.width();
    let image_h = spoke_image.height();

    let file = match File::create("salt.pcm") {
        Ok(f) => f,
        Err(e) => {
            panic!("Can't open file: {}", e)
        }
    };
    let mut writer = BufWriter::with_capacity(2352 * 75 * 100, file);

    let mut rng: SmallRng = rand::make_rng();

    let start_sector: usize = 11387;
    let end_sector: usize = 359840;
    let sector_count: usize = end_sector - start_sector;
    let total_size: usize = sector_count * 2352;

    // println!("Start Position: {:#?}", disc.geometry.polar_at_pos(0.0));
    // println!("End Position: {:#?}", disc.geometry.polar_at_pos(1.0));
    // println!("End Radius: {:#?}", disc.geometry.geometry.outer_radius_mm());

    let bar = ProgressBar::new(total_size as u64);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes}")
        .unwrap()
    );
    for sector in start_sector..end_sector {
        let head = sector * 2352;
        let mut chunk = [0u8; 2352];

        for i in 0..2352 {
            let (r, theta) = disc.be_bin_to_polar_coord((head + i) as u64).unwrap();
            let (sin_theta, cos_theta) = theta.sin_cos();
            let x = cos_theta * r;
            let y = sin_theta * r;
            let x = (x + 60.0) / 120.0 * image_w as f64;
            let y = (y + 60.0) / 120.0 * image_h as f64;
            let x = (x.round() as i32).clamp(0, image_w as i32 - 1) as u32;
            let y = (y.round() as i32).clamp(0, image_h as i32 - 1) as u32;
            let luma: u8 = spoke_image.get_pixel(x, y).0[0];
        
            if rng.random_ratio(luma as u32, 255) {
                chunk[i] = 0xAA;
            } else {
                chunk[i] = 0x20;
            }
        }

        if let Err(e) = writer.write_all(&chunk) {
            panic!("Failed to write to buffer: {}", e)
        }

        bar.inc(2352);

    }
    writer.flush().expect("Failed to flush buffer");
    bar.finish();

    // file.write_all(&bytes).expect("Failed to write.");
}
