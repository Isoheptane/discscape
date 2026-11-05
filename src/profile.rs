use std::fmt;

use crate::geometry::{ExtrapolationMethod, InterpolationMethod};

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
    pub family: DiscFamily,
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
    Interpolation(InterpolationCorrection)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterpolationCorrection {
    pub method: InterpolationMethod,
    pub extrapolation: ExtrapolationMethod,
    pub samples: Vec<[f64; 2]>
}

impl fmt::Display for ExtrapolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtrapolationMethod::Clamp => write!(f, "Clamp"),
        }
    }
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
