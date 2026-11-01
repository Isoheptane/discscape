use crate::geometry::{CalibratedSpiralGeometry, PolarCoord};


pub const MAIN_DATA_SIZE:     usize = 2048;
pub const DATA_FRAME_SIZE:    usize = 2064;

pub const SECTOR_ID_SIZE: usize = 4;
pub const IED_SIZE: usize = 2;
pub const CPR_MAI_SIZE: usize = 6;
pub const EDC_SIZE: usize = 4;

pub const SECTOR_ID_OFFSET: usize = 0;
pub const IED_OFFSET: usize = SECTOR_ID_OFFSET + SECTOR_ID_SIZE;
pub const CPR_MAI_OFFSET: usize = IED_OFFSET + IED_SIZE;
pub const MAIN_DATA_OFFSET: usize = CPR_MAI_OFFSET + CPR_MAI_SIZE;
pub const EDC_OFFSET: usize = MAIN_DATA_OFFSET + MAIN_DATA_SIZE;

pub const SID_INFO_OFFSET: usize = 0;
pub const SID_PSN_OFFSET: usize = 1;
pub const SID_PSN_SIZE: usize = 3;

/* Sector Information */

/*
The sector information bits are
| bit | Meaning | Indicates |
| --- | :- | :- |
| b31 | Sector format type | 1: Zoned format ; 0: CLD format |
| b30 | Tracking method | 1: Groove tracking; 0: Pit tracking |
| b29 | Reflectivity | 1: Reflectivity <= 40%; 0: Reflectivity > 40% | 
| b28 | Reserved | Shall be set to 0 |
| b27 / b26 | Zone Type | 00: Data Zone; 01: Lead-in Zone; 10: Lead-out zone |
| b25 | Data Type | 1: Linking Data; 0: Rewritable / Recordable data;
| b24 | Layer number | 0: Layer 0; 1: Layer 1|
*/

pub const SID_INFO_FORMAT: u8 = 0x80;
pub const SID_INFO_TRACKING: u8 = 0x40;
pub const SID_INFO_REFLECTIVITY: u8 = 0x20;
pub const SID_INFO_RESERVED: u8 = 0x10;
pub const SID_INFO_ZONE_TYPE: u8 = 0x0C;
pub const SID_INFO_DATA_TYPE: u8 = 0x02;
pub const SID_INFO_LAYER: u8 = 0x01;

pub const SID_INFO_FORMAT_CLD: u8 = 0x00;
pub const SID_INFO_FORMAT_ZONED: u8 = SID_INFO_FORMAT;

pub const SID_INFO_TRACKING_PIT: u8 = 0x00;
pub const SID_INFO_TRACKING_GROOVE: u8 = SID_INFO_TRACKING;

pub const SID_INFO_REFLECTIVITY_GT40: u8 = 0x00;
pub const SID_INFO_REFLECTIVITY_LT40: u8 = SID_INFO_REFLECTIVITY;

pub const SID_INFO_ZONE_DATA: u8 = 0x00;
pub const SID_INFO_ZONE_LEAD_IN: u8 = 0x04;
pub const SID_INFO_ZONE_LEAD_OUT: u8 = 0x08;

pub const SID_INFO_DATA_TYPE_LINKING: u8 = SID_INFO_DATA_TYPE;

pub const SID_INFO_LAYER_0: u8 = 0x00;
pub const SID_INFO_LAYER_1: u8 = SID_INFO_LAYER;

/* SCRAMBLER */

pub const SCRAMBLE_LFSR_PRESET_SIZE: usize = 16;
pub const SCRAMBLE_LFSR_INIT: [u16; SCRAMBLE_LFSR_PRESET_SIZE] = [
    0x0001, 0x5500, 0x0002, 0x2A00,
    0x0004, 0x5400, 0x0008, 0x2800,
    0x0010, 0x5000, 0x0020, 0x2001,
    0x0040, 0x4002, 0x0080, 0x0005,
];

pub const SCRAMBLE_LUT: [[u8; MAIN_DATA_SIZE]; SCRAMBLE_LFSR_PRESET_SIZE] = scramble_all_table();

const fn scramble_sequence(init: u16) -> [u8; MAIN_DATA_SIZE] {
    let mut lut = [0u8; MAIN_DATA_SIZE];

    let mut lfsr: u32 = init as u32;
    let mut i = 0;

    while i < MAIN_DATA_SIZE {
        lut[i] = (lfsr & 0xFF) as u8;
        lfsr = (
            (lfsr << 8 & 0x7F00) | 
            ((lfsr >> 7) ^ (lfsr >> 3)) & 0xFF
        ) as u32;

        i += 1;
    }
    lut
}

const fn scramble_all_table() -> [[u8; MAIN_DATA_SIZE]; SCRAMBLE_LFSR_PRESET_SIZE] {
    let mut lut = [[0u8; MAIN_DATA_SIZE]; SCRAMBLE_LFSR_PRESET_SIZE];
    let mut i = 0;

    while i < SCRAMBLE_LFSR_PRESET_SIZE {
        lut[i].copy_from_slice(&scramble_sequence(SCRAMBLE_LFSR_INIT[i]));
        i += 1;
    }

    lut
}

/* Record Frame sizes */
pub const RECORD_FRAME_SIZE: usize = (DATA_FRAME_SIZE / 12 + 10) * 13;

/* EFM Plus Constants */
pub const CHANNEL_WORD_BITS: u64 = 16;
pub const SYNC_CHANNEL_WORDS: u64 = 2;
pub const SYNC_FRAME_BYTES: u64 = 91;
pub const SYNC_FRAME_CHANNEL_WORDS: u64 = 91 + 2;
pub const SYNC_FRAME_PER_SECTOR: u64 = RECORD_FRAME_SIZE as u64 / SYNC_FRAME_BYTES;
pub const PHYSICAL_SECTOR_CHANNEL_WORDS: u64 = SYNC_FRAME_PER_SECTOR * SYNC_FRAME_CHANNEL_WORDS;

#[inline]
const fn user_to_data(pos: u64) -> u64 {
    pos 
        + (pos / MAIN_DATA_SIZE as u64) * (DATA_FRAME_SIZE - MAIN_DATA_SIZE) as u64
        + MAIN_DATA_OFFSET as u64
}

#[inline]
const fn data_to_recording(pos: u64) -> u64 {
    pos 
        + (pos / DATA_FRAME_SIZE as u64) * 182
        + (pos / 172) * 10
}

#[inline]
const fn recording_to_channel_word(pos: u64) -> u64 {
    pos + (pos / 91 + 1) * SYNC_CHANNEL_WORDS
}

#[inline]
const fn user_to_channel_word(pos: u64) -> u64 {
    recording_to_channel_word(data_to_recording(user_to_data(pos)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct DvdDisc {
    geometry: CalibratedSpiralGeometry,
    capacity_sectors: u64,

    total_channel_words: u64,
    inv_total_channel_words: f64,
}

impl DvdDisc {

    pub fn new(geometry: CalibratedSpiralGeometry, sectors: u64) -> Self {
        let total_channel_words = sectors * PHYSICAL_SECTOR_CHANNEL_WORDS;
        let inv_total_channel_words: f64 = 1.0 / total_channel_words as f64;

        Self {
            geometry, 
            capacity_sectors: sectors,
            total_channel_words, 
            inv_total_channel_words
        }
    }

    #[inline] #[allow(unused)] pub fn geometry(&self) -> &CalibratedSpiralGeometry { &self.geometry }
    #[inline] #[allow(unused)] pub fn sectors(&self) -> u64 { self.capacity_sectors }

    pub fn user_to_polar_coord(&self, pos: u64) -> Option<PolarCoord> {
        if pos >= self.capacity_sectors * MAIN_DATA_SIZE as u64 {
            return None;
        }
        let channel_word = user_to_channel_word(pos);
        Some(self.geometry.polar_at_pos(channel_word as f64 * self.inv_total_channel_words))
    }
}