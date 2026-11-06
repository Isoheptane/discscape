use crate::dvd::{DvdDisc, DvdDiscType};
use crate::cd::CdDisc;
use crate::geometry::CorrectedSpiralGeometry;
use crate::profile::{DiscProfile, DiscType, ProfileError};

#[derive(Debug, Clone, PartialEq)]
pub enum Disc {
    Cd(CdDisc),
    Dvd(DvdDisc)
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

impl Disc {
    pub fn sectors(&self) -> u64 {
        match self {
            Disc::Cd(cd_disc) => cd_disc.sectors(),
            Disc::Dvd(dvd_disc) => dvd_disc.sectors(),
        }
    }

    pub fn geometry(&self) -> &CorrectedSpiralGeometry {
        match self {
            Disc::Cd(cd_disc) => cd_disc.geometry(),
            Disc::Dvd(dvd_disc) => dvd_disc.geometry(),
        }
    }

    pub fn with_lut_correction(self, lut_step_mm: f64) -> Self {
        let geometry = self.geometry().clone().with_lut_correction(lut_step_mm);
        match self {
            Disc::Cd(cd_disc) => {
                Self::Cd(CdDisc::new(geometry, cd_disc.sectors()))
            },
            Disc::Dvd(dvd_disc) => {
                Self::Dvd(DvdDisc::new(geometry, *dvd_disc.disc_type(), dvd_disc.sectors()))
            }
        }
    }
}