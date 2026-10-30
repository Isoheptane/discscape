mod geometry;
mod cd;

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use image::{GenericImageView, ImageReader, Pixel};
use indicatif::{ProgressBar, ProgressStyle};
use rand::RngExt;
use rand::rngs::SmallRng;

use crate::{cd::CdDisc, geometry::{CalibratedSpiralGeometry, PhaseCalibration, SpiralGeometry}};




fn main() {

    let geometry = SpiralGeometry {
        r0: 25.0,
        p: 1.509401579,
        n: 22119.863037380
    };

    let calib_geometry = CalibratedSpiralGeometry {
        geometry,
        calibration: PhaseCalibration { samples: vec![] }
    };

    let disc = CdDisc {
        geometry: calib_geometry,
        capacity_sectors: 359_848,
    };

    let spoke_image = ImageReader::open("spoke3.png")
        .expect("Failed to open")
        .decode().expect("Failed to decode")
        .to_luma8();
    let image_w = spoke_image.width();
    let image_h = spoke_image.height();

    let file = match File::create("test.pcm") {
        Ok(f) => f,
        Err(e) => {
            panic!("Can't open file: {}", e)
        }
    };
    let mut writer = BufWriter::with_capacity(2352 * 75 * 100, file);

    let mut rng: SmallRng = rand::make_rng();

    let sector_count: usize = 359840;
    let total_size: usize = sector_count * 2352;

    // println!("Start Position: {:#?}", disc.geometry.polar_at_pos(0.0));
    // println!("End Position: {:#?}", disc.geometry.polar_at_pos(1.0));
    // println!("End Radius: {:#?}", disc.geometry.geometry.outer_radius_mm());

    let bar = ProgressBar::new(total_size as u64);

    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes}")
        .unwrap()
    );
    for sector in 0..sector_count {
        // let head = sector * 2352;
        let mut chunk = [0u8; 2352];

        for i in 0..2352 {
            let (r, theta) = disc.be_bin_to_polar_coord(i as u64).unwrap();
            let x = theta.cos() * r;
            let y = theta.sin() * r;
            let x = (x + 60.0) / 120.0 * image_w as f64;
            let y = (y + 60.0) / 120.0 * image_h as f64;
            let x = (x.round() as i32).clamp(0, image_w as i32 - 1) as u32;
            let y = (y.round() as i32).clamp(0, image_h as i32 - 1) as u32;
            let luma: u8 = spoke_image.get_pixel(x, y).0[0];

            if rng.random_ratio(luma as u32, 256) {
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
