use std::f64::consts::TAU;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use image::ImageReader;
use image::codecs::pnm::ArbitraryTuplType::Grayscale;
use indicatif::{ProgressBar, ProgressStyle};

use discscape::dither::Dither;
use discscape::dvd::{self, DvdDisc, DvdDiscType};
use discscape::geometry::{PhaseCorrectionInterpolation, PhaseSample, SampleInterpolation};
use discscape::geometry::{CalibratedSpiralGeometry, PhaseCorrection, SpiralGeometry};
use discscape::cd::CdDisc;
use discscape::picture::{DiscPicture};

fn main() {
    let start = Instant::now();

    /*
        Next Geometry: 
        SpiralGeometry {
            r0: 24.0,
            p: 0.7406270302613468,
            n: 45961.12875090711,
            p_mm: 0.0007406270302613468,
            total_length: 11845857.329018012,
        }
    */

    let geometry = SpiralGeometry::new(
        24.0,
        0.7406270302613468,
        45961.12875090711,
    );
    println!("{:#?}", geometry);

    let samples = vec![
            PhaseSample { radius_mm: 24.0, correction_rev: 0.000000 },
            PhaseSample { radius_mm: 25.0, correction_rev: 0.001656 },
            PhaseSample { radius_mm: 26.0, correction_rev: 0.002335 },
            PhaseSample { radius_mm: 27.0, correction_rev: 0.003075 },
            PhaseSample { radius_mm: 28.0, correction_rev: 0.003171 },
            PhaseSample { radius_mm: 29.0, correction_rev: 0.003207 },
            PhaseSample { radius_mm: 30.0, correction_rev: 0.002900 },
            PhaseSample { radius_mm: 31.0, correction_rev: 0.002614 },
            PhaseSample { radius_mm: 32.0, correction_rev: 0.002237 },
            PhaseSample { radius_mm: 33.0, correction_rev: 0.001781 },
            PhaseSample { radius_mm: 34.0, correction_rev: 0.002115 },
            PhaseSample { radius_mm: 35.0, correction_rev: 0.001585 },
            PhaseSample { radius_mm: 36.0, correction_rev: 0.001226 },
            PhaseSample { radius_mm: 37.0, correction_rev: 0.000576 },
            PhaseSample { radius_mm: 38.0, correction_rev: 0.000070 },
            PhaseSample { radius_mm: 39.0, correction_rev: 0.000263 },
            PhaseSample { radius_mm: 40.0, correction_rev: 0.000730 },
            PhaseSample { radius_mm: 41.0, correction_rev: 0.001202 },
            PhaseSample { radius_mm: 42.0, correction_rev: 0.001685 },
            PhaseSample { radius_mm: 43.0, correction_rev: 0.001934 },
            PhaseSample { radius_mm: 44.0, correction_rev: 0.002042 },
            PhaseSample { radius_mm: 45.0, correction_rev: 0.002098 },
            PhaseSample { radius_mm: 46.0, correction_rev: 0.002308 },
            PhaseSample { radius_mm: 47.0, correction_rev: 0.002380 },
            PhaseSample { radius_mm: 48.0, correction_rev: 0.002399 },
            PhaseSample { radius_mm: 49.0, correction_rev: 0.002294 },
            PhaseSample { radius_mm: 50.0, correction_rev: 0.002189 },
            PhaseSample { radius_mm: 51.0, correction_rev: 0.002105 },
            PhaseSample { radius_mm: 52.0, correction_rev: 0.001974 },
            PhaseSample { radius_mm: 53.0, correction_rev: 0.001620 },
            PhaseSample { radius_mm: 54.0, correction_rev: 0.001326 },
            PhaseSample { radius_mm: 55.0, correction_rev: 0.000953 },
            PhaseSample { radius_mm: 56.0, correction_rev: 0.000510 },
            PhaseSample { radius_mm: 57.0, correction_rev: 0.000045 },
            PhaseSample { radius_mm: 58.0, correction_rev: 0.000000 },
    ];

    let correction = PhaseCorrectionInterpolation {
        samples,
        interpolation: SampleInterpolation::Linear
    };
    
    let correction = PhaseCorrection::Samples(correction).to_lut(25.0, 58.0, 0.1);
    let correction = PhaseCorrection::Lut(correction);

    // let geometry = discscape::fitter::fit(&geometry, &samples, 58.0).unwrap();
    println!("{:#?}", geometry);

    let calib_geo = CalibratedSpiralGeometry {
        geometry,
        phase_correction: Some(correction)
    };

    

    let disc = DvdDisc::new(calib_geo, DvdDiscType::PlusRw, 2_295_104);

    let image = ImageReader::open("spoke.png")
        .expect("Failed to open")
        .decode().expect("Failed to decode");
    let mut grayscale = image.grayscale();
    grayscale.invert();

    let picture = DiscPicture::new(grayscale);

    let file = match File::create("/tmp/spoke.iso") {
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

    let bar = ProgressBar::new(sector_count as u64 * 2048);

    // bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len} sectors")
    //     .unwrap()
    //     .progress_chars("##.")
    // );
    bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("##.")
    );

    let dither = Dither::splitmix64(0x114514);

    for sector in start_sector..end_sector {

        let chunk = dvd::generate_sector(&disc, &picture, &dither, sector);
        // let chunk = cd::generate_mode2_2336_chunk(&disc, &picture, &mut rng, sector);
        writer.write_all(&chunk).expect("Failed to write to writer");

        bar.inc(2048);

    }
    writer.flush().expect("Failed to flush buffer");
    bar.finish();

    let end = Instant::now();

    let ns = end.duration_since(start).as_nanos();
    println!("Job done in {:.3} seconds", ns as f64 / 1_000_000_000.0)

    
}