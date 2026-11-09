// Generated from the supplied 256-row EFMPlus table.
// Index: [current_state as usize][input_byte as usize].
const EFM_TRANSITIONS: [[u8; 256]; 4] = [
    [
        0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    ],
    [
        1, 0, 1, 3, 1, 1, 2, 0, 2, 2, 3, 3, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 1,
        2, 1, 1, 2, 2, 3, 2, 2, 0, 2, 0, 2, 2, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 0, 2, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 2, 2, 1, 2, 1, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 1, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 0, 2, 0, 0, 0, 0, 0, 1, 1, 1, 2, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0,
        0, 1, 1, 0, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 0, 1, 1, 3, 0, 0, 0, 0, 0,
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 2, 0, 1, 1, 3, 3, 2, 3, 0, 0, 2, 1, 0, 0, 0, 0, 3, 0, 3, 3, 0, 0, 1, 2, 1, 0, 0, 1, 0,
        1, 0, 0, 0, 1, 2, 2, 3, 2, 3, 2, 0, 1, 0, 2, 0, 0, 0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 2, 2, 2, 3, 2, 2, 1, 2, 1, 2, 2, 0,
        0, 0, 2, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 1, 2, 2, 2, 2, 1, 2, 0, 1, 2, 2, 2, 3, 2, 1, 2, 2,
        2, 3, 2, 0, 1, 2, 0, 2, 2, 1, 1, 0, 0, 0, 0, 1, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 1, 2, 2, 2, 1, 1, 2, 1,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 1, 1, 2, 2, 2, 0, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
        0, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    ],
    [
        1, 2, 0, 3, 1, 3, 3, 0, 3, 0, 0, 2, 1, 0, 2, 0, 0, 3, 1, 3, 3, 0, 0, 1, 2, 1, 0, 0, 2, 0,
        2, 0, 0, 0, 1, 3, 2, 3, 0, 3, 0, 0, 1, 0, 2, 3, 0, 0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 2, 2, 2, 3, 2, 2, 1, 2, 1, 2, 2, 0,
        0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 1,
        1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 2, 2, 0, 2, 2, 2, 2, 0, 2, 0, 1, 2, 2, 1, 3, 1, 1, 2, 2,
        2, 3, 2, 0, 1, 2, 0, 0, 3, 1, 1, 2, 1, 2, 0, 1, 1, 1, 2, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 0, 2, 1, 1, 1, 0, 1, 1, 1, 1, 2, 1, 0, 1, 1, 0, 1, 2, 2, 0, 2, 2, 2, 0, 0, 2, 0,
        0, 1, 1, 0, 2, 1, 2, 2, 2, 0, 1, 1, 2, 2, 2, 0, 2, 2, 2, 2, 0, 0, 1, 1, 3, 0, 0, 0, 0, 2,
        2, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EFMPlusState {
    State1 = 0,
    State2 = 1,
    State3 = 2,
    State4 = 3,
}

impl EFMPlusState {
    #[inline]
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::State1),
            1 => Some(Self::State2),
            2 => Some(Self::State3),
            3 => Some(Self::State4),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EFMPlusStateMachine {
    state: EFMPlusState,
}

impl Default for EFMPlusStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl EFMPlusStateMachine {
    #[inline]
    pub fn new() -> Self {
        Self {
            state: EFMPlusState::State1,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.state = EFMPlusState::State1
    }

    #[allow(dead_code)]
    #[inline]
    pub fn set(&mut self, state: EFMPlusState) {
        self.state = state;
    }

    #[inline]
    pub fn next(&mut self, byte: u8) {
        let idx = EFM_TRANSITIONS[self.state as usize][byte as usize];

        // SAFETY CONCERN:
        // EFM_TRANSITIONS is guaranteed to be in 0..=3
        self.state = EFMPlusState::from_index(idx).expect("Invalid EFM+ transition state");
    }

    #[inline]
    pub fn state(self) -> EFMPlusState {
        self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryTransitionDensity {
    LowDensity = 0,
    HighDensity = 1,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EFMPlusStrategy;

impl EFMPlusStrategy {
    const TRANSITION_STRATEGY: [[u8; 2]; 4] = [
        [0x5F, 0x62], // State 1
        // 0000001000000010 ; 0010010010010010
        // 8-8 ; 3-3-3-3-4 ;
        [0x40, 0x62], // State 2
        // 0000010000000010 ; 0010010010010010
        // 8-8 ; 3-3-3-3-4 ;
        [0xB2, 0x70], // State 3
        // 1000000100000010 ; 1001001001001001
        // 7-7-2 ; 3-3-3-3-3 ;
        [0xB2, 0x70], // State 4
        // 1000000100000010 ; 1001001001001001
        // 7-7-2 ; 3-3-3-3-3 ;
    ];

    /*
        This strategy is designed to
        - Keep the state on state 1.
        - Sync incorrectly predicted actual state back to state 1.

        The next state transition table is
        | Actual State | 0x5F | 0x62 | 0x40 | 0xB2 | 0x70 |
        |           1  |   1  |   1  |   1  |   1  |   1  |
        |           2  |   1  |   1  |   1  |   1  |   1  |
        |           3  |   1  |   2  |   2  |   1  |   1  |
        |           4  |   1  |   2  |   2  |   1  |   1  |

        Suppose we represent the transition as
        - (Actual State, Predicted State, Density) -> (Actual State, Predicted State)
        Even if predicted state is not synced with actual state,
        - 1: ({1,2}, {1,2,3,4}, {L/H}) -> (1, 1), synced
        - 2: ({3,4}, {3,4}, {L/H}) -> (1, 1), synced
        - 3: ({3,4}, 1, L) -> (1, 1), synced
        - 4: ({3,4}, 1, H) -> (2, 1), not synced, but will sync on next byte
        - 5: ({3,4}, 2, {L/H}) -> (2, 1), not synced, but will sync on next byte
        
        Funky, isn't it? I even didn't notice this.
    */

    #[inline]
    pub const fn choose(state: EFMPlusState, density: BinaryTransitionDensity) -> u8 {
        Self::TRANSITION_STRATEGY[state as usize][density as usize]
    }
}