pub mod spiral;
pub mod correction;

pub use spiral::{*};

use crate::geometry::correction::PhaseCorrection;

/// Polar coordination of (r, rev)
pub type PolarCoord = (f64, f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseSample {
    pub radius_mm: f64,
    pub correction_rev: f64
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