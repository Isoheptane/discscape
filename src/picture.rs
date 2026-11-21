use image::{DynamicImage, GrayImage};

use crate::geometry::{CartesianCoord, PolarCoord, polar_to_cartesian};

#[derive(Debug, Clone)]
pub struct DiscPicture {
    image: GrayImage,
    width: u32,    
    height: u32,
    scale_x: f64,
    scale_y: f64,
    offset_x: f64,
    offset_y: f64,
}

impl DiscPicture {
    pub fn new(image: DynamicImage) -> Self {
        let luma = image.to_luma8();
        let width = luma.width();
        let height = luma.height();

        let scale_x = width as f64 / 120.0;
        let scale_y = height as f64 / 120.0;

        let offset_x  = width as f64 * 0.5;
        let offset_y  = width as f64 * 0.5;

        Self { image: luma, width, height, scale_x, scale_y, offset_x, offset_y }
    }

    #[inline]
    pub fn sample_cartesian(&self, coord: CartesianCoord) -> u8 {
        let (x_mm, y_mm) = coord;

        let x = x_mm.mul_add(self.scale_x, self.offset_x) as i32;
        let y = y_mm.mul_add(self.scale_y, self.offset_y) as i32;

        let x = x.clamp(0, self.width as i32 - 1) as u32;
        let y = y.clamp(0, self.height as i32 - 1) as u32;
        
        let luma = self.image.get_pixel(x, y).0[0];

        luma
    }

    #[inline]
    pub fn sample_polar(&self, coord: PolarCoord) -> u8 {
        self.sample_cartesian(polar_to_cartesian(coord))
    }
}