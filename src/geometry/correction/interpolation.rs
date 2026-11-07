use std::fmt;

use crate::profile::{CorrectionInterpolationProfile, ProfileError};
use crate::geometry::PhaseSample;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationMethod {
    Linear,
}

impl fmt::Display for InterpolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpolationMethod::Linear => write!(f, "Linear"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtrapolationMethod {
    Zero,
    Clamp,
    Linear,
}

impl fmt::Display for ExtrapolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtrapolationMethod::Zero => write!(f, "Zero"),
            ExtrapolationMethod::Clamp => write!(f, "Clamp"),
            ExtrapolationMethod::Linear => write!(f, "Linear"),
        }
    }
}

/// Use phase samples with interpolation for correction
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseCorrectionInterpolation {
    pub samples: Vec<PhaseSample>,
    pub interpolation: InterpolationMethod,
    pub extrapolation: ExtrapolationMethod,
}

impl PhaseCorrectionInterpolation {
    #[inline]
    fn linear(&self, radius_mm: f64) -> f64 {
        let samples = &self.samples;

        let i = samples.partition_point(|v| v.radius_mm < radius_mm);

        if i == 0 {
            match self.extrapolation {
                ExtrapolationMethod::Zero => return 0.0,
                ExtrapolationMethod::Clamp => return samples[0].rev,
                ExtrapolationMethod::Linear => {
                    let slope = (samples[1].rev - samples[0].rev) 
                        / (samples[1].radius_mm - samples[0].radius_mm);
                    return samples[0].rev + (radius_mm - samples[0].radius_mm) * slope;
                },
            }
        }
        if i == samples.len() {
            match self.extrapolation {
                ExtrapolationMethod::Zero => return 0.0,
                ExtrapolationMethod::Clamp => return samples[i - 1].rev,
                ExtrapolationMethod::Linear => {
                    let slope = (samples[i - 1].rev - samples[i - 2].rev) 
                        / (samples[i - 1].radius_mm - samples[i - 2].radius_mm);
                    return samples[i - 1].rev + (radius_mm - samples[i - 1].radius_mm) * slope;
                },
            }
        }

        let a = samples[i - 1];
        let b = samples[i];
        let t = (radius_mm - a.radius_mm) / (b.radius_mm - a.radius_mm);
        a.rev + t * (b.rev - a.rev)
    }

    #[inline]
    pub fn correction_rev(&self, radius_mm: f64) -> f64 {
        match self.interpolation {
            InterpolationMethod::Linear => self.linear(radius_mm),
            // InterpolationMethod::Pchip => self.pchip(radius_mm),
        }
    }

    pub fn new(
        samples: Vec<PhaseSample>,
        interpolation: InterpolationMethod,
        extrapolation: ExtrapolationMethod,
    ) -> Self {
        let mut samples = samples;
        samples.sort_by(|a, b| a.radius_mm.total_cmp(&b.radius_mm));
        Self {
            samples,
            interpolation,
            extrapolation,
        }
    }
}

impl TryFrom<CorrectionInterpolationProfile> for PhaseCorrectionInterpolation {
    type Error = ProfileError;

    fn try_from(value: CorrectionInterpolationProfile) -> Result<Self, Self::Error> {
        // Check Samples
        let samples: Vec<PhaseSample> = value
            .samples
            .into_iter()
            .map(|v| PhaseSample {
                radius_mm: v[0],
                rev: v[1],
            })
            .collect();

        for (idx, s) in samples.iter().enumerate() {
            if !(s.radius_mm.is_finite() && s.radius_mm > 0.0) {
                return Err(ProfileError::InvalidCorrectionSample {
                    parameter: "radius_mm",
                    value: s.radius_mm,
                    index: idx,
                });
            }
            if !(s.rev.is_finite()) {
                return Err(ProfileError::InvalidCorrectionSample {
                    parameter: "correction_rev",
                    value: s.rev,
                    index: idx,
                });
            }
        }

        if !samples.windows(2).all(|w| w[0].radius_mm < w[1].radius_mm) {
            return Err(ProfileError::InvalidCorrectionSamplesOrder);
        }

        Ok(Self::new(samples, value.method, value.extrapolation))
    }
}
