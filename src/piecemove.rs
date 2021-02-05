use crate::specialmove::SpecialMove;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PieceMove {
    pub start: u8,
    pub end: u8,
    pub special: SpecialMove,
}

impl PieceMove {
    pub fn empty() -> PieceMove {
        PieceMove {
            start: 0,
            end: 0,
            special: SpecialMove::None,
        }
    }
}
