use std::f64::consts::{PI, TAU};

use crate::geometry::PolarCoord;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiralGeometry {
    /// Radius in milimeter
    r0: f64,
    /// Track pitch in micrometer
    p: f64,
    /// Revolution numbers
    n: f64,

    p_mm: f64,
    total_length: f64,
}

impl SpiralGeometry {

    pub fn new(r0_mm: f64, p_um: f64, rev: f64) -> Self {
        let p_mm = p_um * 0.001;
        let total_length = TAU * rev * (r0_mm + p_mm * rev / 2.0);
        Self { r0: r0_mm, p: p_um, n: rev, p_mm, total_length }
    }

    #[inline] pub fn r0_mm(&self) -> f64 { self.r0 }
    #[inline] pub fn pitch_um(&self) -> f64 { self.p }
    #[inline] pub fn pitch_mm(&self) -> f64 { self.p_mm }
    #[inline] pub fn revolutions(&self) -> f64 { self.n }
    #[inline] pub fn total_length(&self) -> f64 { self.total_length }

    pub fn outer_radius_mm(&self) -> f64 {
        self.r0 + (self.n * self.p / 1000.0)
    }

    #[inline]
    pub fn polar_at_pos(&self, pos: f64) -> PolarCoord {
        let length = self.total_length * pos;
        
        let b = TAU * self.r0;
        let a = PI * self.p_mm;
        let rev = 2.0 * length / (b + (b * b + 4.0 * a * length).sqrt());

        let radius_mm = self.r0 + self.p_mm * rev;
        // let angle_rad = TAU * rev.rem_euclid(1.0);

        (radius_mm, rev)
    }

    pub fn pos_at_radius(&self, radius: f64) -> f64 {
        let rev = (radius - self.r0) / self.p_mm;
        let length = TAU * rev * (self.r0 + self.p_mm * rev / 2.0);
        return length / self.total_length;
    }

    pub fn pos_at_rev(&self, rev: f64) -> f64 {
        let theta = rev * TAU;
        let length = theta * (self.r0 + self.p_mm * rev / 2.0);
        return length / self.total_length;
    }

    pub fn pos_at_theta(&self, theta: f64) -> f64 {
        let rev = theta / TAU;
        let length = theta * (self.r0 + self.p_mm * rev / 2.0);
        return length / self.total_length;
    }
}