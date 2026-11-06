use std::fmt;

use crate::geometry::{CorrectedSpiralGeometry, ExtrapolationMethod, InterpolationMethod, PhaseCorrection, SpiralGeometry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiscFamily {
    #[serde(rename = "CD")]
    Cd,
    #[serde(rename = "DVD")]
    Dvd,
}

impl fmt::Display for DiscFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscFamily::Cd => write!(f, "CD"),
            DiscFamily::Dvd => write!(f, "DVD"),
        }
    }
}

impl From<DiscType> for DiscFamily {
    fn from(value: DiscType) -> Self {
        match value {
            DiscType::CdR | DiscType::CdRw => DiscFamily::Cd,
            DiscType::DvdMinusR
            | DiscType::DvdPlusR 
            | DiscType::DvdMinusRw
            | DiscType::DvdPlusRw => DiscFamily::Dvd
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiscType {
    #[serde(rename = "CD-R")]
    CdR,
    #[serde(rename = "CD-RW")]
    CdRw,
    #[serde(rename = "DVD-R")]
    DvdMinusR,
    #[serde(rename = "DVD+R")]
    DvdPlusR,
    #[serde(rename = "DVD-RW")]
    DvdMinusRw,
    #[serde(rename = "DVD+RW")]
    DvdPlusRw,
}

impl fmt::Display for DiscType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscType::CdR => write!(f, "CD-R"),
            DiscType::CdRw => write!(f, "CD-RW"),
            DiscType::DvdMinusR => write!(f, "DVD-R"),
            DiscType::DvdPlusR => write!(f, "DVD+R"),
            DiscType::DvdMinusRw => write!(f, "DVD-RW"),
            DiscType::DvdPlusRw => write!(f, "DVD+RW"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscInfo {
    #[serde(rename = "type")]
    pub disc_type: DiscType,
    pub capacity_sectors: u64
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeometryProfile {
    pub r0_mm: f64,
    pub pitch_um: f64,
    pub n: f64
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CorrectionProfile {
    Interpolation(CorrectionInterpolationProfile)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorrectionInterpolationProfile {
    pub method: InterpolationMethod,
    pub extrapolation: ExtrapolationMethod,
    pub samples: Vec<[f64; 2]>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscProfile {
    pub disc: DiscInfo,
    pub geometry: GeometryProfile,
    pub correction: Option<CorrectionProfile>,
    pub metadata: Option<ProfileMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileMetadata {
    // TODO: Fill profile metadata
    pub media: Option<MediaMetadata>,
    pub recording: Option<RecordingMetadata>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaMetadata {
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub brand: Option<String>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub rated_speed: Option<String>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub mid: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingMetadata {
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub drive: Option<String>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub firmware: Option<String>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub speed_x: Option<f64>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub software: Option<String>,
    #[serde(deserialize_with = "deserialize_option_or_none", default)]
    pub date: Option<String>,
}

fn deserialize_option_or_none<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(T::deserialize(deserializer).ok())
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("Invalid geometry parameter '{parameter}' ({value})")]
    InvalidGeometry { parameter: &'static str, value: f64 },
    #[error("Invalid correction sample '{parameter}' ({value}) at index {index}")]
    InvalidCorrectionSample { parameter: &'static str, value: f64, index: usize },
    #[error("Correction sample radii must be strictly increasing")]
    InvalidCorrectionSamplesOrder,
    #[error("Insufficient correction samples {count}, {required} required")]
    InsufficientCorrectionSamples { count: usize, required: usize },
    #[error("Invalid disc type {disc_type} for DVD")]
    InvalidDvdDiscType { disc_type: DiscType },
}

impl DiscProfile {
    pub fn build_geometry(&self) -> Result<CorrectedSpiralGeometry, ProfileError> {
        let base_geometry = SpiralGeometry::try_from(self.geometry.clone())?;
        let correction = match self.correction.clone() {
            Some(correction) => Some(PhaseCorrection::try_from(correction)?),
            None => None,
        };
        let lut_correction = correction.map(|correction| {
            let r0 = base_geometry.r0_mm();
            let rend = base_geometry.outer_radius_mm();
            // TODO: Maybe require user input step later
            let lut = correction.to_lut(r0, rend, 0.1);
            PhaseCorrection::Lut(lut)
        });
        Ok(CorrectedSpiralGeometry {
            geometry: base_geometry,
            phase_correction: lut_correction,
        })
    }
}