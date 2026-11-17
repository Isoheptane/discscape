#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Pattern {
    Random          = 255,
    PeriodShortest  = 0,
    PeriodShort     = 1,
    PeriodMedium    = 2,
    PeriodLong      = 3,
}