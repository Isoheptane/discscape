#[inline]
pub const fn splitmix64_hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Random {
    Scramble,

    /// Space Address Based Hash (SplitMix64)
    SplitMix64 {
        xor_seed: u64,
    },
}

impl Random {
    pub const fn scramble() -> Self { Self::Scramble }

    pub const fn splitmix64(seed: u64) -> Self {
        Self::SplitMix64 {
            xor_seed: splitmix64_hash(seed)
        }
    }

    /// Returns scrambed sample byte
    #[inline]
    pub const fn sample(&self, address: u64, scramble_byte: u8) -> u8 {
        match self {
            Random::Scramble => scramble_byte,
            Random::SplitMix64 { xor_seed } => {
                (splitmix64_hash(address ^ *xor_seed) & 0xFF) as u8
            },
        }
    }
}