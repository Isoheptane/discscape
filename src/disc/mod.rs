pub mod cd;
pub mod dvd;

pub use cd::{*};
pub use dvd::{*};

use crate::geometry::CorrectedSpiralGeometry;

#[derive(Debug, Clone, PartialEq)]
pub enum Disc {
    Cd(CdDisc),
    Dvd(DvdDisc)
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