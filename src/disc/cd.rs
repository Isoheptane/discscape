use crate::disc::pattern::Pattern;
use crate::dither::Dither;
use crate::picture::DiscPicture;
use crate::geometry::{CorrectedSpiralGeometry, PolarCoord, SpiralGeometry};
use crate::random::Random;

pub const SECTOR_SIZE_BYTES:        usize = 2352;
pub const SYNC_SIZE_BYTES:      usize = 12;
pub const SCRAMBLE_LUT_LEN:         usize = SECTOR_SIZE_BYTES - SYNC_SIZE_BYTES;

pub const F1_FRAME_SIZE_BYTES:      usize = 24;
pub const FRAMES_PER_SECTOR:        usize = SECTOR_SIZE_BYTES / F1_FRAME_SIZE_BYTES;
pub const CHANNEL_BITS_PER_FRAME:   u64 = 588;

pub const AUDIO_CHUNK_SIZE:         usize = 2352;
pub const MODE1_2048_CHUNK_SIZE:    usize = 2048;
pub const MODE1_2352_CHUNK_SIZE:    usize = 2352;
pub const MODE2_2336_CHUNK_SIZE:    usize = 2336;

/// 10010001001001
const EFM_BYTE_SHORT_PERIOD:    u8 = 0xAA;
/// 00000000100000
const EFM_BYTE_LONG_PERIOD:     u8 = 0x20;

pub type FramePosition = (i64, u8);

/// The mapping table for F1 to F3 frame mapping
pub const F1_TO_F3_POS_LUT: [u8; F1_FRAME_SIZE_BYTES] = [
     0,  1,  6,  7, 12, 13, 18, 19,
     2,  3,  8,  9, 14, 15, 20, 21,
     4,  5, 10, 11, 16, 17, 22, 23,
];

pub const F1_TO_F3_DELAY_LUT: [i64; F1_FRAME_SIZE_BYTES] = [
      3,   6,  27,  30,  65,  68,  89,  92,
     11,  14,  35,  38,  73,  76,  97, 100,
     19,  22,  43,  46,  81,  84, 105, 108,
];

/// The mapping table for F3 to F1 frame mapping; .0 is position de-map and .1 is delay frame count
pub const F3_TO_F1_POS_LUT: [u8; F1_FRAME_SIZE_BYTES] = [
     0,  1,  8,  9, 16, 17,  2,  3,
    10, 11, 18, 19,  4,  5, 12, 13,
    20, 21,  6,  7, 14, 15, 22, 23,
];

pub const F3_TO_F1_DELAY_LUT: [i64; F1_FRAME_SIZE_BYTES] = [
      3,   6,  11,  14,  19,  22,  27,  30,
     35,  38,  43,  46,  65,  68,  73,  76,
     81,  84,  89,  92,  97, 100, 105, 108,
];

/// F3 frame position to slot in 33 byte frame
pub const F3_TO_SLOT_MAP: [u8;  F1_FRAME_SIZE_BYTES] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28
];

pub static SCRAMBLE_TABLE: [u8; SCRAMBLE_LUT_LEN] = generate_scramble_table();

const fn generate_scramble_table() -> [u8; SCRAMBLE_LUT_LEN] {
    let mut table = [0u8; SCRAMBLE_LUT_LEN];
    let mut lfsr: u32 = 0x0001;
    let mut i = 0;

    while i < SCRAMBLE_LUT_LEN {
        table[i as usize] = (lfsr & 0xFF) as u8;
        let feedback = (lfsr ^ (lfsr >> 1)) & 0xFF;
        lfsr = (feedback << 7 ^ (lfsr >> 8)) & 0x7FFF;
        i +=1;
    }
    table
}

/// Maps F1 frame to F3 frame position
fn f1_to_f3(pos: FramePosition) -> Option<FramePosition> {
    let (frame, byte) = pos;
    let &pos = F1_TO_F3_POS_LUT.get(byte as usize)?;
    let &delay = F1_TO_F3_DELAY_LUT.get(byte as usize)?;

    Some((frame.checked_add(delay)?, pos))
}

/// Maps F3 frame to F1 frame position
#[allow(dead_code)]
fn f3_to_f1(pos: FramePosition) -> Option<FramePosition> {
    let (frame, byte) = pos;
    let &pos = F3_TO_F1_POS_LUT.get(byte as usize)?;
    let &delay = F3_TO_F1_DELAY_LUT.get(byte as usize)?;

    Some((frame.checked_sub(delay)?, pos))
}

fn be_bin_to_f3(idx: u64) -> Option<FramePosition> {
    let frame = (idx / F1_FRAME_SIZE_BYTES as u64) as i64;
    let byte = (idx % F1_FRAME_SIZE_BYTES as u64) as u8;

    f1_to_f3((frame, byte))
}

fn le_bin_to_f3(idx: u64) -> Option<FramePosition> {
    let frame = (idx / F1_FRAME_SIZE_BYTES as u64) as i64;
    let byte = ((idx % F1_FRAME_SIZE_BYTES as u64) as u8) ^ 1;

    f1_to_f3((frame, byte))
}

#[allow(dead_code)]
fn f3_to_be_bin(pos: FramePosition) -> Option<u64> {
    let (frame, byte) = f3_to_f1(pos)?;
    u64::try_from(frame).ok()?
        .checked_mul(F1_FRAME_SIZE_BYTES as u64)?
        .checked_add(byte as u64)
}

#[allow(dead_code)]
fn f3_to_le_bin(pos: FramePosition) -> Option<u64> {
    let (frame, byte) = f3_to_f1(pos)?;
    u64::try_from(frame).ok()?
        .checked_mul(F1_FRAME_SIZE_BYTES as u64)?
        .checked_add((byte ^ 1) as u64)
}

#[derive(Debug, Clone, PartialEq)]
pub struct CdDisc {
    pub geometry: CorrectedSpiralGeometry,
    capacity_sectors: u64,

    total_frames: u64,
    total_channel_bits: u64,
    inv_total_bits: f64,
}

impl CdDisc {

    pub fn new(geometry: CorrectedSpiralGeometry, sectors: u64) -> Self {
        let total_frames = sectors * FRAMES_PER_SECTOR as u64;
        let total_bits = total_frames * CHANNEL_BITS_PER_FRAME;

        Self {
            geometry, 
            capacity_sectors: sectors,
            total_frames: total_frames,
            total_channel_bits: total_bits,
            inv_total_bits: 1.0 / total_bits as f64
        }
    }

    #[inline] pub fn geometry(&self) -> &CorrectedSpiralGeometry { &self.geometry }
    #[inline] pub fn sectors(&self) -> u64 { self.capacity_sectors }

    pub fn from_cdpaint (
        r0: f64,
        tr0: f64,
        dtr: f64,
        sectors: u64
    ) -> Self {
        let capacity = sectors as f64 * SECTOR_SIZE_BYTES as f64;
        // tr0 is the averave byte count of first revolution, eff_tr0 have an offset of half revolutions
        let eff_tr0 = tr0 - dtr / 2.0;
        let p = 1000.0 * r0 * dtr / eff_tr0;
        // Continuous equivalent revolution count
        let n = 2.0 * capacity / (eff_tr0 + (eff_tr0 * eff_tr0 + 2.0 * dtr * capacity).sqrt());
        let geometry = SpiralGeometry::new(r0, p, n);
        let calib_geometry = CorrectedSpiralGeometry { 
            geometry, 
            phase_correction: None
        };
    
        Self::new(calib_geometry, sectors)
    }

    pub fn f3_to_polar_coord(&self, pos: FramePosition) -> Option<PolarCoord> {
        let (frame, byte) = pos;
        if frame < 0 {
            return None;
        }
        let frame = frame as u64;

        if frame >= self.total_frames {
            return None;
        }
        
        let bit =
            frame * CHANNEL_BITS_PER_FRAME
            + 27
            + *F3_TO_SLOT_MAP.get(byte as usize)? as u64 * 17
            + 7;
        
        Some(self.geometry.polar_at_pos(bit as f64 * self.inv_total_bits))
    }

    pub fn be_bin_to_polar_coord(&self, index: u64) -> Option<PolarCoord> {
        self.f3_to_polar_coord(be_bin_to_f3(index)?)
    }

    pub fn le_bin_to_polar_coord(&self, index: u64) -> Option<PolarCoord> {
        self.f3_to_polar_coord(le_bin_to_f3(index)?)
    }
}

#[inline]
fn u8_to_bcd(val: u8) -> u8 {
    let tens = val / 10;
    let units = val % 10;
    (tens << 4) | units
}

#[inline]
fn sample_image_luma(
    disc: &CdDisc,
    image: &DiscPicture,
    byte_idx: u64,
    cdrom: bool
) -> u8 {
    let coord = if cdrom {
        disc.le_bin_to_polar_coord(byte_idx).expect("Invalid CD-ROM byte index for polar mapping")
    } else {
        disc.be_bin_to_polar_coord(byte_idx).expect("Invalid audio byte index for polar mapping")
    };

    return image.sample_polar(coord);
}

/// Generates a full audio sector
pub fn generate_audio_chunk(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    random: &Random,
    light: Pattern,
    dark: Pattern,
    sector_num: usize,
) -> [u8; AUDIO_CHUNK_SIZE] {
    let mut chunk = [0; AUDIO_CHUNK_SIZE];

    let sector_base = sector_num * SECTOR_SIZE_BYTES;
    for i in 0..AUDIO_CHUNK_SIZE {
        let abs_address = sector_base as u64 + i as u64;
        let luma = sample_image_luma(disc, picture, abs_address, false);
        let pattern = if dither.dither(luma, abs_address) {
            light
        } else {
            dark
        };

        let next_byte = match pattern {
            Pattern::ShortPeriod => {
                EFM_BYTE_SHORT_PERIOD
            },
            Pattern::LongPeriod => {
                EFM_BYTE_LONG_PERIOD
            },
            Pattern::Random => {
                random.sample(abs_address, 0x00)
            },
        };
        
        chunk[i] = next_byte;
    }

    chunk
}

/// Generates a RAW MODE1/2352 chunk, but L-EC information is filled with image data
pub fn generate_mode1_2352_chunk(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    random: &Random,
    light: Pattern,
    dark: Pattern,
    sector_num: usize,
) -> [u8; MODE1_2352_CHUNK_SIZE] {
    let mut chunk = [0; MODE1_2352_CHUNK_SIZE];

    let msf = sector_num + 150;
    let sec = msf / 75;
    let min = sec / 60;

    chunk[0..12].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    chunk[12] = u8_to_bcd(min as u8);
    chunk[13] = u8_to_bcd((sec % 60) as u8);
    chunk[14] = u8_to_bcd((msf % 75) as u8);
    chunk[15] = 0x01;

    let sector_base = sector_num * SECTOR_SIZE_BYTES;
    for i in 16..MODE1_2352_CHUNK_SIZE {
        let abs_address = sector_base as u64 + i as u64;
        let luma = sample_image_luma(disc, picture, abs_address, false);
        let pattern = if dither.dither(luma, abs_address) {
            light
        } else {
            dark
        };

        let next_byte = match pattern {
            Pattern::ShortPeriod => {
                EFM_BYTE_SHORT_PERIOD
            },
            Pattern::LongPeriod => {
                EFM_BYTE_LONG_PERIOD
            },
            Pattern::Random => {
                random.sample(abs_address, SCRAMBLE_TABLE[i - 12])
            },
        };
        
        chunk[i] = next_byte ^ SCRAMBLE_TABLE[i - 12];
    }

    chunk
}

/// Generates a RAW MODE1/2048 chunk
pub fn generate_mode1_2048_chunk(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    random: &Random,
    light: Pattern,
    dark: Pattern,
    sector_num: usize,
) -> [u8; MODE1_2048_CHUNK_SIZE] {
    let mut chunk = [0; MODE1_2048_CHUNK_SIZE];

    let sector_base = sector_num * SECTOR_SIZE_BYTES;
    for i in 0..MODE1_2048_CHUNK_SIZE {
        let abs_address = sector_base as u64 + i as u64 + 16;
        let luma = sample_image_luma(disc, picture, abs_address, false);
        let pattern = if dither.dither(luma, abs_address) {
            light
        } else {
            dark
        };

        let next_byte = match pattern {
            Pattern::ShortPeriod => {
                EFM_BYTE_SHORT_PERIOD
            },
            Pattern::LongPeriod => {
                EFM_BYTE_LONG_PERIOD
            },
            Pattern::Random => {
                random.sample(abs_address, SCRAMBLE_TABLE[i + 4])
            },
        };
        
        chunk[i] = next_byte ^ SCRAMBLE_TABLE[i + 4];
    }

    chunk
}

/// Generates a RAW MODE2/2336 chunk
pub fn generate_mode2_2336_chunk(
    disc: &CdDisc,
    picture: &DiscPicture,
    dither: &Dither,
    random: &Random,
    light: Pattern,
    dark: Pattern,
    sector_num: usize,
) -> [u8; MODE2_2336_CHUNK_SIZE] {
    let mut chunk = [0; MODE2_2336_CHUNK_SIZE];

    let sector_base = sector_num * SECTOR_SIZE_BYTES;
    for i in 0..MODE2_2336_CHUNK_SIZE {
        let abs_address = sector_base as u64 + i as u64 + 16;
        let luma = sample_image_luma(disc, picture, abs_address, false);
        let pattern = if dither.dither(luma, abs_address) {
            light
        } else {
            dark
        };

        let next_byte = match pattern {
            Pattern::ShortPeriod => {
                EFM_BYTE_SHORT_PERIOD
            },
            Pattern::LongPeriod => {
                EFM_BYTE_LONG_PERIOD
            },
            Pattern::Random => {
                random.sample(abs_address, SCRAMBLE_TABLE[i + 4])
            },
        };
        
        chunk[i] = next_byte ^ SCRAMBLE_TABLE[i + 4];
    }
    chunk
}