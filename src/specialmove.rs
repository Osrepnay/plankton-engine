#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
