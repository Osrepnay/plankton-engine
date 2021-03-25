use crate::specialmove::SpecialMove;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PieceMove {
    pub start: u8,
    pub end: u8,
    pub special: SpecialMove,
}

impl Default for PieceMove {
    fn default() -> Self {
        PieceMove {
            start: 0,
            end: 0,
            special: SpecialMove::None,
        }
    }
}
