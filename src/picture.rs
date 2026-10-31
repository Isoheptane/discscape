use image::{DynamicImage, GrayImage};

use crate::geometry::PolarCoord;

pub struct DiscPicture {
    image: GrayImage,
    width: u32,    
    height: u32,
    scale_x: f64,
    scale_y: f64,
}

impl DiscPicture {
    pub fn new(image: DynamicImage) -> Self {
        let luma = image.to_luma8();
        let width = luma.width();
        let height = luma.height();

        let scale_x = width as f64 / 120.0;
        let scale_y = height as f64 / 120.0;

        Self { image: luma, width, height, scale_x, scale_y }
    }

    #[inline]
    pub fn sample_polar(&self, coord: PolarCoord) -> u8 {
        let (r, rev) = coord;

        let (sin_theta, cos_theta) = sin_cos_rev(rev);
        let x = cos_theta.mul_add(r, 60.0).mul_add( self.scale_x, 0.5) as i32;
        let y = sin_theta.mul_add(r, 60.0).mul_add( self.scale_y, 0.5) as i32;

        let x = x.clamp(0, self.width as i32 - 1) as u32;
        let y = y.clamp(0, self.height as i32 - 1) as u32;
        
        self.image.get_pixel(x, y).0[0]
    }
}

use crate::sin_table::{*};

#[inline(always)]
pub fn sin_cos_rev(rev: f64) -> (f64, f64) {
    let phase = rev - rev.floor();

    let pos = phase * LUT_SIZE as f64;
    let idx = pos as usize;
    let t = pos - idx as f64;

    let s0 = SIN_LUT[idx];
    let s1 = SIN_LUT[idx + 1];
    let c0 = SIN_LUT[idx + LUT_QUARTER];
    let c1 = SIN_LUT[idx + LUT_QUARTER + 1];

    let sin = (s1 - s0).mul_add(t, s0);
    let cos = (c1 - c0).mul_add(t, c0);

    (sin, cos)
}