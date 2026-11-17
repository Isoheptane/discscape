#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum BasePeriod {
    Shortest  = 0,
    Short     = 1,
    Medium    = 2,
    Long      = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Pattern {
    PeriodShortest  = 0,    // 6:0 -> AAAAAA (DVD 3.20T, 30 flips) 
    MacroX5S1       = 1,    // 5:1 -> AAABAA (DVD 3.31T, 29 flips) 
    MacroX4S2       = 2,    // 4:2 -> AABAAB (DVD 3.43T, 28 flips) 
    MacroX3S3       = 3,    // 3:3 -> ABABAB (DVD 3.55T, 27 flips) 
    MacroX2S4       = 4,    // 2:4 -> BBABBA (DVD 3.69T, 26 flips)
    MacroX1S5       = 5,    // 1:5 -> BBBABB (DVD 3.84T, 25 flips)

    PeriodShort     = 6,    // 6:0 -> AAAAAA (DVD 4.00T, 24 flips) 
    MacroS5M1       = 7,    // 5:1 -> AAABAA (DVD 4.17T, 23 flips) 
    MacroS4M2       = 8,    // 4:2 -> AABAAB (DVD 4.36T, 22 flips) 
    MacroS3M3       = 9,    // 3:3 -> ABABAB (DVD 4.57T, 21 flips) 
    MacroS2M4       = 10,   // 2:4 -> BBABBA (DVD 4.80T, 20 flips)
    MacroS1M5       = 11,   // 1:5 -> BBBABB (DVD 5.05T, 19 flips)

    PeriodMedium    = 12,   // 6:0 -> AAAAAA (DVD 5.33T, 18 flips) 
    MacroM5L1       = 13,   // 5:1 -> AAABAA (DVD 5.65T, 17 flips) 
    MacroM4L2       = 14,   // 4:2 -> AABAAB (DVD 6.00T, 16 flips) 
    MacroM3L3       = 15,   // 3:3 -> ABABAB (DVD 6.40T, 15 flips) 
    MacroM2L4       = 16,   // 2:4 -> BBABBA (DVD 6.86T, 14 flips)
    MacroM1L5       = 17,   // 1:5 -> BBBABB (DVD 7.38T, 13 flips)

    PeriodLong      = 18,   // 6:0 -> AAAAAA (DVD 8.00T, 12 flips) 

    Random          = 255,
}

impl Pattern {
    pub const MACRO_LEN: usize = 6;

    #[inline]
    pub const fn base_period_at(self, sub_idx: usize) -> BasePeriod {
        use BasePeriod::{Shortest as X, Short as S, Medium as M, Long as L};
        const TABLE: [[BasePeriod; 6]; 19] = [
            [X, X, X, X, X, X],
            [X, X, X, S, X, X],
            [X, X, S, X, X, S],
            [X, S, X, S, X, S],
            [S, S, X, S, S, X],
            [S, S, S, X, S, S],

            [S, S, S, S, S, S],
            [S, S, S, M, S, S],
            [S, S, M, S, S, M],
            [S, M, S, M, S, M],
            [M, M, S, M, M, S],
            [M, M, M, S, M, M],

            [M, M, M, M, M, M],
            [M, M, M, L, M, M],
            [M, M, L, M, M, L],
            [M, L, M, L, M, L],
            [L, L, M, L, L, M],
            [L, L, L, M, L, L],

            [L, L, L, L, L, L],
        ];

        let idx = self as usize;
        if idx < 19 {
            TABLE[idx][sub_idx % 6]
        } else {
            BasePeriod::Shortest
        }
    }
}