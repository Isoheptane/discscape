use std::fmt;

use crate::{disc::{CdDisc, Disc, DvdDisc, DvdDiscType}, geometry::{CorrectedSpiralGeometry, PhaseSample, SpiralGeometry, correction::{ExtrapolationMethod, InterpolationMethod, PhaseCorrection, PhaseCorrectionInterpolation}}};

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

impl TryFrom<GeometryProfile> for SpiralGeometry {
    type Error = ProfileError;
    fn try_from(value: GeometryProfile) -> Result<Self, Self::Error> {
        for (parameter, value) in [
            ("r0_mm", value.r0_mm),
            ("pitch_um", value.pitch_um),
            ("n", value.n),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(ProfileError::InvalidGeometry { parameter, value })
            }
        }
        Ok(Self::new(value.r0_mm, value.pitch_um, value.n))
    }
}

/*
    General Correction Profile
*/

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CorrectionProfile {
    Interpolation(CorrectionInterpolationProfile)
}

impl TryFrom<CorrectionProfile> for PhaseCorrection {
    type Error = ProfileError;
    
    fn try_from(value: CorrectionProfile) -> Result<Self, Self::Error> {
        let correction = match value {
            CorrectionProfile::Interpolation(interpo) => {
                PhaseCorrection::Interpolation(PhaseCorrectionInterpolation::try_from(interpo)?)
            }
        };
        Ok(correction)
    }
}

/*
    Interpolation Type Profile
*/

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorrectionInterpolationProfile {
    pub method: InterpolationMethod,
    pub extrapolation: ExtrapolationMethod,
    pub samples: Vec<[f64; 2]>
}

impl TryFrom<CorrectionInterpolationProfile> for PhaseCorrectionInterpolation {
    type Error = ProfileError;
    
    fn try_from(value: CorrectionInterpolationProfile) -> Result<Self, Self::Error> {
        // Check samples
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

        Ok(PhaseCorrectionInterpolation::new(
            samples, value.method, value.extrapolation
        ))
    }
}

/*
    Whole Disc Profile
*/

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscProfile {
    pub disc: DiscInfo,
    pub geometry: GeometryProfile,
    pub correction: Option<CorrectionProfile>,
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

impl TryFrom<DiscProfile> for Disc {
    type Error = ProfileError;
    
    fn try_from(profile: DiscProfile) -> Result<Self, Self::Error> {
        let geometry = profile.build_geometry()?;

        match profile.disc.disc_type {
            DiscType::CdR | DiscType::CdRw => {
                Ok(Disc::Cd(CdDisc::new(
                    geometry, profile.disc.capacity_sectors
                )))
            }
            disc_type => {
                let dvd_type = DvdDiscType::try_from(disc_type)?;

                Ok(Disc::Dvd(DvdDisc::new(
                    geometry,
                    dvd_type,
                    profile.disc.capacity_sectors
                )))
            }
        }
    }
    
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("Profile have invalid geometry parameter '{parameter}' ({value})")]
    InvalidGeometry { parameter: &'static str, value: f64 },
    #[error("Profile have invalid correction sample '{parameter}' ({value}) at index {index}")]
    InvalidCorrectionSample { parameter: &'static str, value: f64, index: usize },
    #[error("Correction sample radii must be strictly increasing in profile")]
    InvalidCorrectionSamplesOrder,
    #[error("Profile have insufficient correction samples {count}, {required} required")]
    InsufficientCorrectionSamples { count: usize, required: usize },
    #[error("Profile have invalid disc type {disc_type} for DVD")]
    InvalidDvdDiscType { disc_type: DiscType },
}

impl DiscProfile {
    pub fn build_geometry(&self) -> Result<CorrectedSpiralGeometry, ProfileError> {
        let base_geometry = SpiralGeometry::try_from(self.geometry.clone())?;
        let correction = self.correction
            .clone()
            .map(PhaseCorrection::try_from)
            .transpose()?;
        Ok(CorrectedSpiralGeometry {
            geometry: base_geometry,
            phase_correction: correction,
        })
    }
}