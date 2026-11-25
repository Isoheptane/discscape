use std::fmt;

use crate::geometry::PhaseSample;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationMethod {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "cubic_bessel")]
    CubicBessel
}

impl fmt::Display for InterpolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpolationMethod::Linear => write!(f, "Linear"),
            InterpolationMethod::CubicBessel => write!(f, "Cubic Bessel"),
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

    fn derivative_left(&self) -> f64 {
        match self.interpolation {
            InterpolationMethod::Linear => {
                (self.samples[1].rev - self.samples[0].rev) /
                (self.samples[1].radius_mm - self.samples[0].radius_mm)
            },
            InterpolationMethod::CubicBessel => {
                Self::cubic_hermite_derivative(&self, 0)
            }
        }
    }

    fn derivative_right(&self) -> f64 {
        let end = self.samples.len() - 1;
        match self.interpolation {
            InterpolationMethod::Linear => {
                (self.samples[end].rev - self.samples[end - 1].rev) /
                (self.samples[end].radius_mm - self.samples[end - 1].radius_mm)
            },
            InterpolationMethod::CubicBessel => {
                Self::cubic_hermite_derivative(&self, end)
            }
        }
    }

    fn extrapolate_left(&self, radius_mm: f64) -> f64 {
        if radius_mm == self.samples[0].radius_mm {
            return self.samples[0].rev;
        }
        match self.extrapolation {
            ExtrapolationMethod::Zero => return 0.0,
            ExtrapolationMethod::Clamp => return self.samples[0].rev,
            ExtrapolationMethod::Linear => {
                let slope = self.derivative_left();
                return self.samples[0].rev + (radius_mm - self.samples[0].radius_mm) * slope;
            }
        }
    }

    fn extrapolate_right(&self, radius_mm: f64) -> f64 {
        let end = self.samples.len() - 1;
        match self.extrapolation {
            ExtrapolationMethod::Zero => return 0.0,
            ExtrapolationMethod::Clamp => return self.samples[end].rev,
            ExtrapolationMethod::Linear => {
                let slope = self.derivative_right();
                return self.samples[end].rev + (radius_mm - self.samples[end].radius_mm) * slope;
            }
        }
    }

    #[inline]
    fn cubic_hermite_derivative(&self, i: usize) -> f64 {
        let samples = &self.samples;
        let len = samples.len();

        if len == 2 {
            return (samples[1].rev - samples[0].rev) / (samples[1].radius_mm - samples[0].radius_mm);
        }

        let inner_deriv = |i: usize| -> f64 {
            let hl = samples[i].radius_mm - samples[i - 1].radius_mm;
            let hr = samples[i + 1].radius_mm - samples[i].radius_mm;
            let sl = (samples[i].rev - samples[i - 1].rev) / hl;
            let sr = (samples[i + 1].rev - samples[i].rev) / hr;
            (hr * sl + hl * sr) / (hl + hr)
        };
        if i == 0 {
            let s = 
                (samples[1].rev - samples[0].rev) / 
                (samples[1].radius_mm - samples[0].radius_mm);
            return 1.5 * s - 0.5 * inner_deriv(1);
        } else if i == len - 1 {
            let s = 
                (samples[len - 1].rev - samples[len - 2].rev) / 
                (samples[len - 1].radius_mm - samples[len - 2].radius_mm);
            return 1.5 * s - 0.5 * inner_deriv(len - 2)
        } else {
            return inner_deriv(i)
        }
    }

    #[inline]
    fn cubic_bessel(&self, radius_mm: f64) -> f64 {
        let samples = &self.samples;

        let i = samples.partition_point(|v| v.radius_mm < radius_mm);

        if i == 0 {
            return self.extrapolate_left(radius_mm);
        } else if i == samples.len() {
            return self.extrapolate_right(radius_mm)
        }

        let h = samples[i].radius_mm - samples[i - 1].radius_mm;
        let s = (samples[i].rev - samples[i - 1].rev) / h;
        let dl = self.cubic_hermite_derivative(i - 1);
        let dr = self.cubic_hermite_derivative(i);

        let dx = radius_mm - samples[i - 1].radius_mm;
        let t = dx / h;
        let c2 = 3.0 * s - 2.0 * dl - dr;
        let c3 = dl + dr - 2.0 * s;

        samples[i - 1].rev + dx * (dl + t * (c2 + t * c3))
    }

    #[inline]
    fn linear(&self, radius_mm: f64) -> f64 {
        let samples = &self.samples;

        let i = samples.partition_point(|v| v.radius_mm < radius_mm);

        if i == 0 {
            return self.extrapolate_left(radius_mm);
        } else if i == samples.len() {
            return self.extrapolate_right(radius_mm)
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
            InterpolationMethod::CubicBessel => self.cubic_bessel(radius_mm)
        }
    }

    pub fn new(
        samples: Vec<PhaseSample>,
        interpolation: InterpolationMethod,
        extrapolation: ExtrapolationMethod,
    ) -> Self {

        assert!(samples.len() >= 2);

        let mut samples = samples;
        samples.sort_by(|a, b| a.radius_mm.total_cmp(&b.radius_mm));
        Self {
            samples,
            interpolation,
            extrapolation,
        }
    }
}