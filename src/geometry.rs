pub type PolarCoord = (f64, f64);

use std::f64::consts::{PI, TAU};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiralGeometry {
    /// Radius in milimeter
    pub r0: f64,
    /// Track pitch in micrometer
    pub p: f64,
    /// Revolution numbers
    pub n: f64
}

impl SpiralGeometry {
    pub fn outer_radius_mm(&self) -> f64 {
        self.r0 + (self.n * self.p / 1000.0)
    }
    pub fn polar_at_pos(&self, pos: f64) -> Option<PolarCoord> {
        let p_mm = self.p / 1000.0;
        let total_length = TAU * self.n * (self.r0 + p_mm * self.n / 2.0);
        let length = total_length * pos;

        let b = TAU * self.r0;
        let a = PI * p_mm;
        let rev = 2.0 * length / (b + (b * b + 4.0 * a * length).sqrt());

        let radius_mm = self.r0 + p_mm * rev;
        // let angle_rad = TAU * rev.rem_euclid(1.0);

        Some((radius_mm, rev * TAU))
    }
    #[allow(unused)]
    pub fn pos_at_radius(&self, radius: f64) -> f64 {
        let p_mm = self.p / 1000.0;
        let total_length = TAU * self.n * (self.r0 + p_mm * self.n / 2.0);
        let rev = (radius - self.r0) / p_mm;
        let length = TAU * rev * (self.r0 + p_mm * rev / 2.0);
        return length / total_length;
    }
    #[allow(unused)]
    pub fn pos_at_theta(&self, theta: f64) -> f64 {
        let p_mm = self.p / 1000.0;
        let total_length = TAU * self.n * (self.r0 + p_mm * self.n / 2.0);
        let rev = theta / TAU;
        let length = theta * (self.r0 + p_mm * rev / 2.0);
        return length / total_length;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseSample {
    pub radius_mm: f64,
    pub correction_deg: f64
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhaseCalibration {
    pub samples: Vec<PhaseSample>,
}

impl PhaseCalibration {
    pub fn correction_rad(&self, radius_mm: f64) -> f64 {
        let samples = &self.samples;
        if samples.is_empty() {
            return 0.0;
        }
        let i = samples.partition_point(|v| v.radius_mm < radius_mm);
        if i == 0 {
            return samples[0].correction_deg.to_radians();
        }
        if i == samples.len() {
            return samples[i - 1].correction_deg.to_radians();
        }
        let a = samples[i - 1];
        let b = samples[i];
        let t = (radius_mm - a.radius_mm) / (b.radius_mm - a.radius_mm);
        let degrees = a.correction_deg + t * (b.correction_deg - a.correction_deg);
        degrees.to_radians()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibratedSpiralGeometry {
    /// Radius in milimeter
    pub geometry: SpiralGeometry,
    /// Calibration Data
    pub calibration: PhaseCalibration
}

impl CalibratedSpiralGeometry {
    pub fn polar_at_pos(&self, pos: f64) -> Option<PolarCoord> {
        let (r, theta) = self.geometry.polar_at_pos(pos)?;

        Some((r, theta + self.calibration.correction_rad(r)))
    }
}