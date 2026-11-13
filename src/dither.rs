use crate::random::splitmix64_hash;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dither {
    /// No dithering, use fixed gate.
    /// Any value greater than threshold will return 1
    Gate { threshold: u8 },

    /// Space Address Based Hash (SplitMix64)
    SplitMix64 {
        xor_seed: u64,
    },
}

impl Dither {
    #[inline]
    const fn gate_dither(threshold: u8, luma: u8) -> bool {
        return luma >= threshold;
    }

    #[inline]
    const fn splitmix64_dither(xor_seed: u64, address: u64, luma: u8) -> bool {
        let hash = splitmix64_hash(address ^ xor_seed);
        let threshold = ((hash as u128 * 255) >> 64) as u8;
        luma > threshold
    }

    #[inline]
    pub fn dither(&self, luma: u8, address: u64) -> bool {
        match self {
            Dither::Gate { threshold } => {
                Dither::gate_dither(*threshold, luma)
            },
            Dither::SplitMix64 { xor_seed, .. } => {
                Dither::splitmix64_dither(*xor_seed, address, luma)
            }
        }
    }

    pub const fn gate(threshold: u8) -> Self { Self::Gate { threshold } }

    pub const fn splitmix64(seed: u64) -> Self {
        Self::SplitMix64 {
            xor_seed: splitmix64_hash(seed)
        }
    }
}