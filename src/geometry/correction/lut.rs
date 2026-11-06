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