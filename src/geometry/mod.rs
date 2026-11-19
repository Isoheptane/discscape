pub mod spiral;
pub mod correction;

pub use spiral::{*};

use crate::geometry::correction::PhaseCorrection;
use crate::sin_table::{SIN_LUT_QUARTER, SIN_LUT_SIZE, SIN_LUT};

/// Polar coordination of (r, rev)
pub type PolarCoord = (f64, f64);
pub type CartesianCoord = (f64, f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseSample {
    pub radius_mm: f64,
    pub rev: f64
}

#[derive(Debug, Clone, PartialEq)]
pub struct CorrectedSpiralGeometry {
    /// Radius in milimeter
    pub geometry: SpiralGeometry,
    /// Calibration Data
    pub phase_correction: Option<PhaseCorrection>
}

impl CorrectedSpiralGeometry {
    #[inline]
    pub fn polar_at_pos(&self, pos: f64) -> PolarCoord {
        let (r, rev) = self.geometry.polar_at_pos(pos);
        let rev_correction = self.phase_correction.as_ref()
            .map_or(0.0, |p| p.correction_rev(r));

        (r, rev + rev_correction)
    }

    pub fn with_lut_correction(self, lut_step_mm: f64) -> Self {
        let phase_correction = self.phase_correction.map(|correction| {
            let r0 = self.geometry.r0_mm();
            let rend = self.geometry.outer_radius_mm();
            PhaseCorrection::Lut(correction.to_lut(r0, rend, lut_step_mm))
        });

        Self {
            geometry: self.geometry,
            phase_correction,
        }
    }
}

#[inline(always)]
pub fn polar_to_cartesian(coord: PolarCoord) -> CartesianCoord {
    let (r, rev) = coord;
    let (sin_theta, cos_theta) = sin_cos_rev(rev);
    (r * cos_theta, r * sin_theta)
}

#[inline(always)]
pub fn sin_cos_rev(rev: f64) -> (f64, f64) {
    let phase = rev - rev.floor();

    let pos = phase * SIN_LUT_SIZE as f64;
    let idx = pos as usize;
    let t = pos - idx as f64;

    let s0 = SIN_LUT[idx];
    let s1 = SIN_LUT[idx + 1];
    let c0 = SIN_LUT[idx + SIN_LUT_QUARTER];
    let c1 = SIN_LUT[idx + SIN_LUT_QUARTER + 1];

    let sin = (s1 - s0).mul_add(t, s0);
    let cos = (c1 - c0).mul_add(t, c0);

    (sin, cos)
}