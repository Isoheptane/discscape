/*
    The Scrambler uses 16 presets. Each 16 sector uses one preset and continue.
*/

use crate::disc::dvd::MAIN_DATA_SIZE;

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