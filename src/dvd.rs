use rand::{RngExt, rngs::SmallRng};

use crate::{efmplus::{EFMPlusStateMachine, EFMPlusStrategy, TransitionDensity}, geometry::{CalibratedSpiralGeometry, PolarCoord}, picture::DiscPicture};


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
        lut[i] = scramble_sequence(SCRAMBLE_LFSR_INIT[i]);
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

/* Sector Information Byte Default Value */
pub const DVD_MINUS_R_SID_INFO_DEFAULT: u8 = 0x00;
pub const DVD_MINUS_RW_SID_INFO_DEFAULT: u8 = 0x20;
pub const DVD_PLUS_R_SID_INFO_DEFAULT: u8 = 0x00;
pub const DVD_PLUS_RW_SID_INFO_DEFAULT: u8 = 0x62;

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
const fn user_to_recording(pos: u64) -> u64 {
    data_to_recording(user_to_data(pos))
}

#[inline]
const fn user_to_channel_word(pos: u64) -> u64 {
    recording_to_channel_word(data_to_recording(user_to_data(pos)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DvdDiscType {
    PlusR,
    PlusRw,
    MinusR,
    MinusRw,
}

impl DvdDiscType {
    pub fn default_sector_id_info_byte(&self) -> u8{
        match self {
            DvdDiscType::PlusR => DVD_PLUS_R_SID_INFO_DEFAULT,
            DvdDiscType::PlusRw => DVD_PLUS_RW_SID_INFO_DEFAULT,
            DvdDiscType::MinusR => DVD_MINUS_R_SID_INFO_DEFAULT,
            DvdDiscType::MinusRw => DVD_MINUS_RW_SID_INFO_DEFAULT,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DvdDisc {
    geometry: CalibratedSpiralGeometry,
    disc_type: DvdDiscType,
    capacity_sectors: u64,

    total_channel_words: u64,
    inv_total_channel_words: f64,
}

impl DvdDisc {

    pub fn new(geometry: CalibratedSpiralGeometry, disc_type: DvdDiscType, sectors: u64) -> Self {
        let total_channel_words = sectors * PHYSICAL_SECTOR_CHANNEL_WORDS;
        let inv_total_channel_words: f64 = 1.0 / total_channel_words as f64;

        Self {
            geometry, 
            disc_type,
            capacity_sectors: sectors,
            total_channel_words, 
            inv_total_channel_words
        }
    }

    #[inline] #[allow(unused)] pub fn geometry(&self) -> &CalibratedSpiralGeometry { &self.geometry }
    #[inline] #[allow(unused)] pub fn disc_type(&self) -> &DvdDiscType { &self.disc_type }
    #[inline] #[allow(unused)] pub fn sectors(&self) -> u64 { self.capacity_sectors }

    pub fn user_to_polar_coord(&self, pos: u64) -> Option<PolarCoord> {
        if pos >= self.capacity_sectors * MAIN_DATA_SIZE as u64 {
            return None;
        }
        let channel_word = user_to_channel_word(pos);
        Some(self.geometry.polar_at_pos(channel_word as f64 * self.inv_total_channel_words))
    }

    pub fn channel_word_to_polar_coord(&self, pos: u64) -> Option<PolarCoord> {
        if pos >= self.total_channel_words {
            return None;
        }
        Some(self.geometry.polar_at_pos(pos as f64 * self.inv_total_channel_words))
    }
}

pub fn generate_sector(
    disc: &DvdDisc,
    image: &DiscPicture,
    rng: &mut SmallRng,
    sector_num: usize,
) -> [u8; MAIN_DATA_SIZE] {
    // Set EFM Plus state machine by 12 bytes
    let mut state = EFMPlusStateMachine::new();
    // Sector ID
    state.next(disc.disc_type().default_sector_id_info_byte());
    let psn = sector_num as u64 + 0x30000;
    state.next(((psn >> 16) & 0xFF) as u8);
    state.next(((psn >> 8) & 0xFF) as u8);
    state.next((psn & 0xFF) as u8);
    // Initialize scramble table
    let scramble_table = &SCRAMBLE_LUT[((psn & 0xF0) >> 4) as usize];

    /*
        TODO: Finish IED Calculation
        Without IED calculation, the EFM Plus state can not be correctly predicted for first recording frame.
    */

    state.next(0x00);
    state.next(0x00);
    // CPR_MAI
    for _ in 0..6 { state.next(0x00) }
    
    // Initialize image luma sampling first
    const CHANNEL_WORDS_PER_IMAGE_SAMPLE: usize = 6;
    const IMAGE_SAMPLES_LEN: usize = PHYSICAL_SECTOR_CHANNEL_WORDS as usize / CHANNEL_WORDS_PER_IMAGE_SAMPLE;
    assert_eq!(PHYSICAL_SECTOR_CHANNEL_WORDS as usize % CHANNEL_WORDS_PER_IMAGE_SAMPLE, 0);

    // 2418 = 2 x 3 x 13 x 31, take one sample per 6 channel words

    let mut image_samples = [0u8; IMAGE_SAMPLES_LEN];

    for i in 0..IMAGE_SAMPLES_LEN {
        let mid_word = i * CHANNEL_WORDS_PER_IMAGE_SAMPLE + (CHANNEL_WORDS_PER_IMAGE_SAMPLE / 2);
        let abs_pos = 
            sector_num as u64
            * PHYSICAL_SECTOR_CHANNEL_WORDS as u64
            + mid_word as u64;
        
        let coord = disc.channel_word_to_polar_coord(abs_pos).expect("Absolute value out of range");
        let luma = image.sample_polar(coord);
        image_samples[i] = luma;
    }
    let image_samples = image_samples;

    // Start byte sampling
    let mut chunk = [0u8; MAIN_DATA_SIZE];

    for i in 0..MAIN_DATA_SIZE {
        let channel_word = user_to_channel_word(i as u64);
        // Frame head reset event, immediately after 2 word sync
        if channel_word % 93 == 2 {
            state.reset();
        }

        let luma = image_samples[channel_word as usize / CHANNEL_WORDS_PER_IMAGE_SAMPLE];
        let next_byte = if rng.random_ratio(luma as u32, 255) {
            EFMPlusStrategy::choose(state.state(), TransitionDensity::HighDensity)
        } else {
            EFMPlusStrategy::choose(state.state(), TransitionDensity::LowDensity)
        };
        state.next(next_byte);

        chunk[i] = next_byte ^ scramble_table[i]
    }

    chunk
}