use std::fmt::{self, write};

use crate::profile::ExtrapolationType::Clamp;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DiscType {
    #[serde(rename = "CD-R")]
    CdRr,
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
            DiscType::CdRr => write!(f, "CD-R"),
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
    pub extrapolation: ExtrapolationType,
    pub samples: Vec<[f64; 2]>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationMethod {
    Linear
}

impl fmt::Display for InterpolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpolationMethod::Linear => write!(f, "Linear"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtrapolationType {
    Clamp
}

impl fmt::Display for ExtrapolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtrapolationType::Clamp => write!(f, "Clamp"),
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
}