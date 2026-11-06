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
    Clamp,
}

impl fmt::Display for ExtrapolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtrapolationMethod::Clamp => write!(f, "Clamp"),
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
        if samples.is_empty() {
            return 0.0;
        }

        let i = samples.partition_point(|v| v.radius_mm < radius_mm);

        if i == 0 {
            match self.extrapolation {
                ExtrapolationMethod::Clamp => return samples[0].rev,
            }
        }
        if i == samples.len() {
            match self.extrapolation {
                ExtrapolationMethod::Clamp => return samples[i - 1].rev,
            }
        }

        let a = samples[i - 1];
        let b = samples[i];
        let t = (radius_mm - a.radius_mm) / (b.radius_mm - a.radius_mm);
        a.rev + t * (b.rev - a.rev)
    }

    #[allow(dead_code)]
    #[inline]
    fn pchip(&self, radius_mm: f64) -> f64 {
        let samples = &self.samples;
        if samples.len() < 2 {
            return 0.0;
        }
        let i = samples.partition_point(|v| v.radius_mm < radius_mm);

        if i == 0 {
            match self.extrapolation {
                ExtrapolationMethod::Clamp => return samples[0].rev,
            }
        }
        if i == samples.len() {
            match self.extrapolation {
                ExtrapolationMethod::Clamp => return samples[i - 1].rev,
            }
        }

        let x0 = samples[i - 1].radius_mm;
        let x1 = samples[i].radius_mm;
        let y0 = samples[i - 1].rev;
        let y1 = samples[i].rev;
        let h = x1 - x0;
        let t = (radius_mm - x0) / h;

        if samples.len() == 2 {
            return y0 + t * (y1 - y0);
        }

        let secant = |j: usize| -> f64 {
            (samples[j + 1].rev - samples[j].rev)
                / (samples[j + 1].radius_mm - samples[j].radius_mm)
        };

        let deriv = |k: usize| -> f64 {
            if k == 0 {
                return secant(0);
            };
            if k == samples.len() - 1 {
                return secant(k - 1);
            };
            let a = secant(k - 1);
            let b = secant(k);
            if a * b <= 0.0 {
                return 0.0;
            } // Flat derivative for local ext
            let hp = samples[k].radius_mm - samples[k - 1].radius_mm;
            let hn = samples[k + 1].radius_mm - samples[k].radius_mm;
            let w1 = 2.0 * hn + hp;
            let w2 = hn + 2.0 * hp;
            (w1 + w2) / (w1 / a + w2 / b)
        };

        let d0 = deriv(i - 1);
        let d1 = deriv(i);

        let t2 = t * t;
        let t3 = t2 * t;

        y0 * (2.0 * t3 - 3.0 * t2 + 1.0)
            + h * d0 * (t3 - 2.0 * t2 + t)
            + y1 * (-2.0 * t3 + 3.0 * t2)
            + h * d1 * (t3 - t2)
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
