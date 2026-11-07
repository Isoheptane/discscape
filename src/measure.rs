use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::geometry::PhaseSample;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementUnit {
    #[serde(rename = "deg")]
    Degree,
    #[serde(rename = "rev")]
    Revolution,
    #[serde(rename = "arc_mm")]
    ArcMilimeter
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementDoc {
    pub unit: MeasurementUnit,
    pub ref_radius: Option<f64>,
    // [radius_mm, measurement]
    pub samples: Vec<[f64; 2]>,

    #[serde(default)]
    pub base_profile: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl MeasurementDoc {
    pub fn to_phase_samples(&self) -> Vec<PhaseSample> {
        let mut sorted = self.samples.clone();
        // Replace with a error later
        sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).expect("Not comparable"));
        sorted.dedup_by_key(|a| a[0]);

        sorted.into_iter().map(|[r, val]|{
            let rev = match self.unit {
                MeasurementUnit::Degree => val / 360.0,
                MeasurementUnit::Revolution => val,
                MeasurementUnit::ArcMilimeter => val / (TAU * r),
            };
            PhaseSample { radius_mm: r, rev }
        }).collect()
    }
}