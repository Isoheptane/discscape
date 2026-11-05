use std::{f64::consts::{PI, TAU}, fmt};

/// Polar coordination of (r, rev)
pub type PolarCoord = (f64, f64);

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseSample {
    pub radius_mm: f64,
    pub correction_rev: f64
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
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

/// Use phase samples with interpolation for correction
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseCorrectionInterpolation {
    pub samples: Vec<PhaseSample>,
    pub interpolation: InterpolationMethod
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtrapolationMethod {
    Clamp
}

impl PhaseCorrectionInterpolation {

    #[inline]
    fn linear(&self, radius_mm: f64) -> f64 {
        let samples = &self.samples;
        if samples.is_empty() { return 0.0; }
        let i = samples.partition_point(|v| v.radius_mm < radius_mm);
        if i == 0 {
            return samples[0].correction_rev;
        }
        if i == samples.len() {
            return samples[i - 1].correction_rev;
        }
        let a = samples[i - 1];
        let b = samples[i];
        let t = (radius_mm - a.radius_mm) / (b.radius_mm - a.radius_mm);
        a.correction_rev + t * (b.correction_rev - a.correction_rev)
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
            return samples[0].correction_rev;
        }
        if i == samples.len() {
            return samples[i - 1].correction_rev;
        }
        
        let x0 = samples[i - 1].radius_mm;
        let x1 = samples[i].radius_mm;
        let y0 = samples[i - 1].correction_rev;
        let y1 = samples[i].correction_rev;
        let h = x1 - x0;
        let t = (radius_mm - x0) / h;

        if samples.len() == 2 {
            return y0 + t * (y1 - y0);
        }

        let secant = |j: usize| -> f64 {
            (samples[j + 1].correction_rev - samples[j].correction_rev) /
            (samples[j + 1].radius_mm - samples[j].radius_mm)
        };

        let deriv = |k: usize| -> f64 {
            if k == 0 { return secant(0) };
            if k == samples.len() - 1 { return secant(k - 1) };
            let a = secant(k - 1);
            let b = secant(k);
            if a * b <= 0.0 { return 0.0 } // Flat derivative for local ext
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

    pub fn new(samples: Vec<PhaseSample>, interpolation: InterpolationMethod) -> Self {
        let mut samples = samples;
        samples.sort_by(|a, b| a.radius_mm.total_cmp(&b.radius_mm));
        Self { samples, interpolation }
    }
}

/// Lookup table for correction
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseCorrectionLut {
    pub radius_min: f64,
    pub step: f64,
    step_inv: f64,
    pub lut: Vec<f64>
}

impl PhaseCorrectionLut {
    pub fn from_function(
        radius_min: f64, 
        radius_max: f64, 
        step_mm: f64,
        mut correction: impl FnMut(f64) -> f64,
    ) -> Self {
        assert!(radius_max > radius_min);
        assert!(step_mm > 0.0);

        let count = ((radius_max - radius_min) / step_mm).ceil() as usize;

        let actual_step = (radius_max - radius_min) / count as f64;
        let step_inv = 1.0 / actual_step;

        let values: Vec<f64> = (0..=count)
            .map(|i| {
                let r = radius_min + i as f64 * actual_step;
                correction(r)
            })
            .collect();

        PhaseCorrectionLut { radius_min, step: actual_step, step_inv, lut: values }
        
    }

    pub fn correction_rev(&self, radius_mm: f64) -> f64 {
        let max_idx = self.lut.len() - 1;

        let pos = ((radius_mm - self.radius_min) * self.step_inv)
            .clamp(0.0, max_idx as f64);

        let idx = (pos as usize).min(max_idx - 1);
        let t = pos - idx as f64;

        let a = self.lut[idx];
        let b = self.lut[idx + 1];

        return a + (b - a) * t;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhaseCorrection {
    Samples(PhaseCorrectionInterpolation),
    Lut(PhaseCorrectionLut),
}

impl PhaseCorrection {
    #[inline]
    pub fn correction_rev(&self, radius_mm: f64) -> f64 {
        match self {
            PhaseCorrection::Samples(curve) => curve.correction_rev(radius_mm),
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

#[derive(Debug, Clone, PartialEq)]
pub struct CalibratedSpiralGeometry {
    /// Radius in milimeter
    pub geometry: SpiralGeometry,
    /// Calibration Data
    pub phase_correction: Option<PhaseCorrection>
}

impl CalibratedSpiralGeometry {
    #[inline]
    pub fn polar_at_pos(&self, pos: f64) -> PolarCoord {
        let (r, rev) = self.geometry.polar_at_pos(pos);
        let rev_correction = self.phase_correction.as_ref()
            .map_or(0.0, |p| p.correction_rev(r));

        (r, rev + rev_correction)
    }
}