pub mod interpolation;
pub mod lut;

pub use interpolation::{*};
pub use lut::{*};

#[derive(Debug, Clone, PartialEq)]
pub enum PhaseCorrection {
    Interpolation(PhaseCorrectionInterpolation),
    Lut(PhaseCorrectionLut),
}

impl PhaseCorrection {
    #[inline]
    pub fn correction_rev(&self, radius_mm: f64) -> f64 {
        match self {
            PhaseCorrection::Interpolation(curve) => curve.correction_rev(radius_mm),
            PhaseCorrection::Lut(curve) => curve.correction_rev(radius_mm),
        }
    }

    pub fn to_lut(&self, radius_min: f64, radius_max: f64, step_mm: f64) -> PhaseCorrectionLut {
        match self {
            PhaseCorrection::Lut(phase_correction_lut) => phase_correction_lut.clone(),
            _ => PhaseCorrectionLut::from_function(radius_min, radius_max, step_mm, |r| self.correction_rev(r))
        }
    }
}