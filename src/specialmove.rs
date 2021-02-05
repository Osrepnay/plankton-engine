#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SpecialMove {
    KnightPromotion,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    EnPassant,
    CastleKingside,
    CastleQueenside,
    None,
}
