use crate::geometry::{PolarCoord, polar_to_cartesian};
use crate::picture::DiscPicture;
use crate::dither::Dither;
use crate::disc::pattern::Pattern;
use crate::random::splitmix64_hash;

pub trait PatternSampler {
    fn sample(&self, coord: PolarCoord, addr: u64) -> Pattern;

    fn use_random(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct GrayscaleSampler {
    pub picture: DiscPicture,
    pub dither: Dither,
    pub light: Pattern,
    pub dark: Pattern,
    gamma_lut: [u32; 256]
}

impl GrayscaleSampler {
    pub fn new(
        picture: DiscPicture,
        dither: Dither,
        light: Pattern,
        dark: Pattern,
        gamma: f64,
    ) -> Self {
        Self {
            picture,
            dither,
            light,
            dark,
            gamma_lut: build_gamma_lut(gamma, 1.0),
        }
    }
}

impl PatternSampler for GrayscaleSampler {
    #[inline]
    fn sample(&self, coord: PolarCoord, addr: u64) -> Pattern {
        let luma = self.picture.sample_polar(coord);
        let linear_luma = self.gamma_lut[luma as usize];
        if self.dither.dither_u32(linear_luma, addr) {
            self.light
        } else {
            self.dark
        }
    }
    
    fn use_random(&self) -> bool {
        self.light == Pattern::Random || self.dark == Pattern::Random
    }
}

#[derive(Debug, Clone)]
pub struct MultiplexChannel {
    pub pattern: Pattern,
    pub picture: DiscPicture,
    pub gamma_lut: [u32; 256],
}

#[derive(Debug, Clone)]
pub struct MultiplexSampler {
    pub channels: Vec<MultiplexChannel>,
    pub dump: Pattern,
    pub seed: u64,
}

impl MultiplexSampler {
    pub fn new(
        channels: Vec<(Pattern, DiscPicture, f64)>,
        dump: Pattern,
        seed: u64,
        gamma: f64,
    ) -> Self {
        let sum: f64 = channels.iter().map(|(_, _, w)| w).sum();
        let norm_factor = if sum > 1.0 { 1.0 / sum } else { 1.0 };

        let channels: Vec<MultiplexChannel> = channels.into_iter().map(|(pattern, picture, w)| {
            let normalized_w = w * norm_factor;
            let gamma_lut = build_gamma_lut(gamma, normalized_w);
            MultiplexChannel {
                pattern,
                picture,
                gamma_lut,
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

        let planar = polar_to_cartesian(coord);

        let mut sum: u32 = 0;
        for ch in &self.channels {
            let luma = ch.picture.sample_cartesian(planar);
            sum = sum.saturating_add(ch.gamma_lut[luma as usize]);
            if hash < sum {
                return ch.pattern
            }
        }

        self.dump
    }
    
    fn use_random(&self) -> bool {
        for ch in &self.channels {
            if ch.pattern == Pattern::Random {
                return true;
            }
        }
        false
    }

    
}

fn build_gamma_lut(gamma: f64, scale: f64) -> [u32; 256] {
    let mut lut = [0u32; 256];        

    let max_val = scale * (u32::MAX as f64);

    for (i, slot) in lut.iter_mut().enumerate().skip(1) {
        let norm_luma = i as f64 / 255.0;
        let linear = norm_luma.powf(gamma);
        *slot = (linear * max_val).round().clamp(0.0, u32::MAX as f64) as u32;
    }

    lut
}