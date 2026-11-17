use crate::geometry::PolarCoord;
use crate::picture::DiscPicture;
use crate::dither::Dither;
use crate::disc::pattern::Pattern;
use crate::random::splitmix64_hash;

pub trait PatternSampler {
    fn sample(&self, coord: PolarCoord, addr: u64) -> Pattern;
}

#[derive(Clone)]
pub struct GrayscaleSampler {
    pub picture: DiscPicture,
    pub dither: Dither,
    pub light: Pattern,
    pub dark: Pattern
}

impl PatternSampler for GrayscaleSampler {
    #[inline]
    fn sample(&self, coord: PolarCoord, addr: u64) -> Pattern {
        let luma = self.picture.sample_polar(coord);
        if self.dither.dither(luma, addr) {
            self.light
        } else {
            self.dark
        }
    }
}

#[derive(Clone)]
pub struct MultiplexChannel {
    pub pattern: Pattern,
    pub picture: DiscPicture,
    pub weight_per_luma: u32,
}

#[derive(Clone)]
pub struct MultiplexSampler {
    pub channels: Vec<MultiplexChannel>,
    pub dump: Pattern,
    pub seed: u64,
}

impl MultiplexSampler {
    pub fn new(
        channels: Vec<(Pattern, DiscPicture, f64)>,
        dump: Pattern,
        seed: u64
    ) -> Self {
        let sum: f64 = channels.iter().map(|(_, _, w)| w).sum();
        let norm_factor = if sum > 1.0 { 1.0 / sum } else { 1.0 };

        let channels: Vec<MultiplexChannel> = channels.into_iter().map(|(pattern, picture, w)| {
            let normalized_w = w * norm_factor;
            let step_luma = (normalized_w * ((u32::MAX as f64) / 255.0)).round() as u32;
            MultiplexChannel {
                pattern,
                picture,
                weight_per_luma: step_luma
            }
        })
        .collect();

        Self { channels, dump, seed }
    }
}

impl PatternSampler for MultiplexSampler {
    #[inline]
    fn sample(&self, coord: PolarCoord, addr: u64) -> Pattern {
        let hash = (splitmix64_hash(addr ^ self.seed) >> 32) as u32;

        let mut sum: u32 = 0;
        for ch in &self.channels {
            let luma = ch.picture.sample_polar(coord) as u32;
            sum = sum.saturating_add(luma * ch.weight_per_luma);
            if hash < sum {
                return ch.pattern
            }
        }

        self.dump
    }
}