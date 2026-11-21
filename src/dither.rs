use crate::random::splitmix64_hash;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dither {
    xor_seed: u64,
}

impl Dither {
    #[inline]
    const fn splitmix64_dither_u8(xor_seed: u64, address: u64, luma: u8) -> bool {
        let hash = splitmix64_hash(address ^ xor_seed);
        let threshold = ((hash as u128 * u8::MAX as u128) >> 64) as u8;
        luma > threshold
    }

    #[inline]
    const fn splitmix64_dither_u16(xor_seed: u64, address: u64, luma: u16) -> bool {
        let hash = splitmix64_hash(address ^ xor_seed);
        let threshold = ((hash as u128 * u16::MAX as u128) >> 64) as u16;
        luma > threshold
    }

    #[inline]
    const fn splitmix64_dither_u32(xor_seed: u64, address: u64, luma: u32) -> bool {
        let hash = splitmix64_hash(address ^ xor_seed);
        let threshold = ((hash as u128 * u32::MAX as u128) >> 64) as u32;
        luma > threshold
    }

    #[inline]
    pub fn dither_u8(&self, luma: u8, address: u64) -> bool {
        Dither::splitmix64_dither_u8(self.xor_seed, address, luma)
    }

    #[inline]
    pub fn dither_u16(&self, luma: u16, address: u64) -> bool {
        Dither::splitmix64_dither_u16(self.xor_seed, address, luma)
    }

    #[inline]
    pub fn dither_u32(&self, luma: u32, address: u64) -> bool {
        Dither::splitmix64_dither_u32(self.xor_seed, address, luma)
    }

    pub const fn new(seed: u64) -> Self {
        Self {
            xor_seed: splitmix64_hash(seed)
        }
    }
}