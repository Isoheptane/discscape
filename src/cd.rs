use crate::geometry::{CalibratedSpiralGeometry, PhaseCalibration, PolarCoord, SpiralGeometry};

pub const BYTES_PER_F1: u64 = 24;
pub const BYTES_PER_SECTOR: u64 = 2352;
pub const FRAMES_PER_SECTOR: u64 = BYTES_PER_SECTOR / BYTES_PER_F1;
pub const CHANNEL_BITS_PER_FRAME: u64 = 588;

pub type FramePosition = (i64, u8);

/// The mapping table for F1 to F3 frame mapping
pub const F1_TO_F3_POS_LUT: [u8; 24] = [
     0,  1,  6,  7, 12, 13, 18, 19,
     2,  3,  8,  9, 14, 15, 20, 21,
     4,  5, 10, 11, 16, 17, 22, 23,
];

pub const F1_TO_F3_DELAY_LUT: [i64; 24] = [
      3,   6,  27,  30,  65,  68,  89,  92,
     11,  14,  35,  38,  73,  76,  97, 100,
     19,  22,  43,  46,  81,  84, 105, 108,
];

/// The mapping table for F3 to F1 frame mapping; .0 is position de-map and .1 is delay frame count
#[allow(unused)]
pub const F3_TO_F1_POS_LUT: [u8; 24] = [
     0,  1,  8,  9, 16, 17,  2,  3,
    10, 11, 18, 19,  4,  5, 12, 13,
    20, 21,  6,  7, 14, 15, 22, 23,
];
#[allow(unused)]
pub const F3_TO_F1_DELAY_LUT: [i64; 24] = [
      3,   6,  11,  14,  19,  22,  27,  30,
     35,  38,  43,  46,  65,  68,  73,  76,
     81,  84,  89,  92,  97, 100, 105, 108,
];

pub fn f1_to_f3(pos: FramePosition) -> Option<FramePosition> {
    let (frame, byte) = pos;
    let &pos = F1_TO_F3_POS_LUT.get(byte as usize)?;
    let &delay = F1_TO_F3_DELAY_LUT.get(byte as usize)?;

    Some((frame.checked_add(delay)?, pos))
}
#[allow(unused)]
pub fn f3_to_f1(pos: FramePosition) -> Option<FramePosition> {
    let (frame, byte) = pos;
    let &pos = F3_TO_F1_POS_LUT.get(byte as usize)?;
    let &delay = F3_TO_F1_DELAY_LUT.get(byte as usize)?;

    Some((frame.checked_sub(delay)?, pos))
}

pub fn be_bin_to_f3(idx: u64) -> Option<FramePosition> {
    let frame = (idx / BYTES_PER_F1) as i64;
    let byte = (idx % BYTES_PER_F1) as u8;

    f1_to_f3((frame, byte))
}
#[allow(unused)]
pub fn le_bin_to_f3(idx: u64) -> Option<FramePosition> {
    let frame = (idx / BYTES_PER_F1) as i64;
    let byte = ((idx % BYTES_PER_F1) as u8) ^ 1;

    f1_to_f3((frame, byte))
}
#[allow(unused)]
pub fn f3_to_be_bin(pos: FramePosition) -> Option<u64> {
    let (frame, byte) = f3_to_f1(pos)?;
    u64::try_from(frame).ok()?
        .checked_mul(BYTES_PER_F1)?
        .checked_add(byte as u64)
}
#[allow(unused)]
pub fn f3_to_le_bin(pos: FramePosition) -> Option<u64> {
    let (frame, byte) = f3_to_f1(pos)?;
    u64::try_from(frame).ok()?
        .checked_mul(BYTES_PER_F1)?
        .checked_add((byte ^ 1) as u64)
}
#[allow(unused)]
pub fn f3_audio_slot(byte: u8) -> Option<u64> {
    match byte {
        0..=11 => Some((byte + 1) as u64),
        12..=23 => Some((byte + 5) as u64),
        _ => None
    }
}

pub const F3_TO_SLOT_MAP: [u8; 24] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28
];

#[derive(Debug, Clone, PartialEq)]
pub struct CdDisc {
    pub geometry: CalibratedSpiralGeometry,
    pub capacity_sectors: u64,
}

impl CdDisc {
    #[allow(unused)]
    pub fn from_cdpaint (
        r0: f64,
        tr0: f64,
        dtr: f64,
        sectors: u64
    ) -> Self {
        let capacity = sectors as f64 * 2352.0;
        // tr0 is the averave byte count of first revolution, eff_tr0 have an offset of half revolutions
        let eff_tr0 = tr0 - dtr / 2.0;
        let p = 1000.0 * r0 * dtr / eff_tr0;
        // Continuous equivalent revolution count
        let n = 2.0 * capacity / (eff_tr0 + (eff_tr0 * eff_tr0 + 2.0 * dtr * capacity).sqrt());
        let geometry = SpiralGeometry { r0, p, n };
        let calib_geometry = CalibratedSpiralGeometry { geometry, calibration: PhaseCalibration::default() };

        CdDisc { geometry: calib_geometry, capacity_sectors: sectors }
    }

    pub fn f3_to_polar_coord(&self, pos: FramePosition) -> Option<PolarCoord> {
        let (frame, byte) = pos;
        if frame < 0 {
            return None;
        }
        let frame = frame as u64;
        let total_frames = self.capacity_sectors * FRAMES_PER_SECTOR;
        if frame >= total_frames {
            return None;
        }

        let total_bits = total_frames * CHANNEL_BITS_PER_FRAME;
        let bit =
            frame * CHANNEL_BITS_PER_FRAME
            + 27
            + *F3_TO_SLOT_MAP.get(byte as usize)? as u64 * 17
            + 7;
        
        self.geometry.polar_at_pos(bit as f64 / total_bits as f64)
    }

    pub fn be_bin_to_polar_coord(&self, index: u64) -> Option<PolarCoord> {
        self.f3_to_polar_coord(be_bin_to_f3(index)?)
    }

    #[allow(unused)]
    pub fn le_bin_to_polar_coord(&self, index: u64) -> Option<PolarCoord> {
        self.f3_to_polar_coord(le_bin_to_f3(index)?)
    }
}