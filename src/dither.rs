
#[derive(Debug, Clone)]
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

#[inline]
const fn splitmix64_hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}