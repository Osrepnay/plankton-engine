use crate::moveutil;
use crate::piecemove::PieceMove;
use crate::specialmove::SpecialMove;

#[derive(Clone, PartialEq)]
pub struct MoveGen {
    rays: [[u64; 8]; 64],
    knight_moves: [u64; 64],
    king_moves: [u64; 64],
}

impl MoveGen {
    pub fn new() -> MoveGen {
        let mut knight_moves = [0; 64];
        for (idx, knight_move) in knight_moves.iter_mut().enumerate() {
            let idx = idx as i64;
            let possible_knight_moves = [
                idx + 15,
                idx + 17,
                idx + 10,
                idx - 6,
                idx - 15,
                idx - 17,
                idx - 10,
                idx + 6,
            ];
            for possible_knight_move in &possible_knight_moves {
                let possible_knight_move = *possible_knight_move;
                if possible_knight_move >= 0 && possible_knight_move < 64 {
                    //i have no idea how this works anymore
                    if !((idx - 1) % 8 == 0 && (possible_knight_move + 1) % 8 == 0)
                        && !(idx % 8 == 0
                            && ((possible_knight_move + 2) % 8 == 0
                                || (possible_knight_move + 1) % 8 == 0))
                        && !((idx + 2) % 8 == 0 && possible_knight_move % 8 == 0)
                        && !((idx + 1) % 8 == 0
                            && ((possible_knight_move - 1) % 8 == 0
                                || possible_knight_move % 8 == 0))
                    {
                        *knight_move |= 1u64 << possible_knight_move as u64;
                    }
                }
            }
        }

        let mut king_moves = [0; 64];
        for (idx, king_move) in king_moves.iter_mut().enumerate() {
            let idx = idx as i64;
            let possible_king_moves = [
                idx + 8,
                idx + 9,
                idx + 1,
                idx - 7,
                idx - 8,
                idx - 9,
                idx - 1,
                idx + 7,
            ];
            for possible_king_move in &possible_king_moves {
                let possible_king_move = *possible_king_move;
                if possible_king_move >= 0 && possible_king_move < 64 {
                    if !(idx % 8 == 0 && (possible_king_move + 1) % 8 == 0)
                        && !((idx + 1) % 8 == 0 && possible_king_move % 8 == 0)
                    {
                        *king_move |= 1u64 << possible_king_move as u64;
                    }
                }
            }
        }
        fn gen_ray(position: u8, direction: u8) -> u64 {
            let mut ray = 0;
            match direction {
                0 => {
                    // up
                    for idx in (position..64).step_by(8) {
                        ray |= 1u64 << idx;
                    }
                }
                1 => {
                    // up-right
                    for idx in (position..64).step_by(9) {
                        if idx % 8 == 0 && idx != position {
                            break;
                        }
                        ray |= 1u64 << idx;
                    }
                }
                2 => {
                    // right
                    for idx in position..64 {
                        if idx % 8 == 0 && idx != position {
                            break;
                        }
                        ray |= 1u64 << idx;
                    }
                }
                3 => {
                    // down-right
                    for idx in ((position % 7)..(position + 1)).step_by(7).rev() {
                        if idx % 8 == 0 && idx != position {
                            break;
                        }
                        ray |= 1u64 << idx;
                    }
                }
                4 => {
                    //down
                    for idx in ((position % 8)..(position + 1)).step_by(8).rev() {
                        ray |= 1u64 << idx;
                    }
                }
                5 => {
                    // down-left
                    for idx in ((position % 9)..(position + 1)).step_by(9).rev() {
                        if (idx + 1) % 8 == 0 && idx != position {
                            break;
                        }
                        ray |= 1u64 << idx;
                    }
                }
                6 => {
                    // left
                    for idx in (0..(position + 1)).rev() {
                        if (idx + 1) % 8 == 0 && idx != position {
                            break;
                        }
                        ray |= 1u64 << idx;
                    }
                }
                7 => {
                    // up-left
                    for idx in (position..64).step_by(7) {
                        if (idx + 1) % 8 == 0 && idx != position {
                            break;
                        }
                        ray |= 1u64 << idx;
                    }
                }
                _ => (),
            };
            ray & !(1u64 << position)
        }
        let mut rays = [[0; 8]; 64];
        for (position, square_rays) in rays.iter_mut().enumerate() {
            for (direction, ray) in square_rays.iter_mut().enumerate() {
                *ray = gen_ray(position as u8, direction as u8);
            }
        }
        MoveGen {
            rays,
            knight_moves,
            king_moves,
        }
    }
    pub fn gen_move(
        &self,
        color: u8,
        piece: u8,
        position: u8,
        blockers: u64,
        castle_available: [bool; 4],
    ) -> ([PieceMove; 28], u8) {
        match piece {
            0 => self.gen_pawn(color, position, blockers),
            1 => (
                moveutil::bitboard_to_piecemoves(self.knight_moves[position as usize], position),
                self.knight_moves[position as usize].count_ones() as u8,
            ),
            2 => self.gen_bishop(position, blockers),
            3 => self.gen_rook(position, blockers),
            4 => self.gen_queen(position, blockers),
            5 => self.gen_king(color, position, blockers, castle_available),
            _ => ([PieceMove::empty(); 28], 0),
        }
    }
    fn gen_pawn(&self, color: u8, position: u8, blockers: u64) -> ([PieceMove; 28], u8) {
        let position = position as i8;
        let mut square_moves = [PieceMove::empty(); 28];
        let mut num_moves: u8 = 0;
        let pos_change = -((color as i8 * 2 - 1) * 8);
        let is_promotion = position + pos_change >= 56 || position + pos_change < 8;
        let mut add_move = |start: u8, end: u8| {
            if is_promotion {
                let promotions = [
                    SpecialMove::KnightPromotion,
                    SpecialMove::BishopPromotion,
                    SpecialMove::RookPromotion,
                    SpecialMove::QueenPromotion,
                ];
                for promotion in &promotions {
                    square_moves[num_moves as usize] = PieceMove {
                        start,
                        end,
                        special: *promotion,
                    };
                    num_moves += 1;
                }
            } else {
                square_moves[num_moves as usize] = PieceMove {
                    start,
                    end,
                    special: SpecialMove::None,
                };
                num_moves += 1;
            }
        };
        if ((blockers >> (position + pos_change)) & 1) == 0 {
            add_move(position as u8, (position + pos_change) as u8);
            if color == 0 {
                if position >= 8
                    && position < 16
                    && ((blockers >> (position + 2 * pos_change)) & 1) == 0
                {
                    add_move(position as u8, (position + 2 * pos_change) as u8);
                }
            } else {
                if position >= 48
                    && position < 56
                    && ((blockers >> (position + 2 * pos_change)) & 1) == 0
                {
                    add_move(position as u8, (position + 2 * pos_change) as u8);
                }
            }
        }
        if position + pos_change + 1 >= 0
            && position + pos_change + 1 < 64
            && ((blockers >> (position + pos_change + 1)) & 1) != 0
            && (position + 1) % 8 != 0
        {
            add_move(position as u8, (position + pos_change + 1) as u8);
        }
        if position + pos_change - 1 >= 0
            && position + pos_change - 1 < 64
            && ((blockers >> (position + pos_change - 1)) & 1) != 0
            && position % 8 != 0
        {
            add_move(position as u8, (position + pos_change - 1) as u8);
        }
        (square_moves, num_moves)
    }
    fn gen_bishop(&self, position: u8, blockers: u64) -> ([PieceMove; 28], u8) {
        let mut board = 0;
        for i in 0..4 {
            let masked_blockers = blockers & self.rays[position as usize][i * 2 + 1];
            let moves = {
                if masked_blockers == 0 {
                    self.rays[position as usize][i * 2 + 1]
                } else {
                    let first_blocker_pos = {
                        match i * 2 + 1 {
                            1 => masked_blockers.trailing_zeros(),
                            7 => masked_blockers.trailing_zeros(),
                            3 => 63 - masked_blockers.leading_zeros(),
                            5 => 63 - masked_blockers.leading_zeros(),
                            _ => 0,
                        }
                    };
                    let blocker_ray = self.rays[first_blocker_pos as usize][i * 2 + 1];
                    self.rays[position as usize][i * 2 + 1] & !blocker_ray
                }
            };
            board |= moves;
        }
        (
            moveutil::bitboard_to_piecemoves(board, position),
            board.count_ones() as u8,
        )
    }
    fn gen_rook(&self, position: u8, blockers: u64) -> ([PieceMove; 28], u8) {
        let mut board = 0;
        for i in 0..4 {
            let masked_blockers = blockers & self.rays[position as usize][i * 2];
            let moves = {
                if masked_blockers == 0 {
                    self.rays[position as usize][i * 2]
                } else {
                    let first_blocker_pos = {
                        match i * 2 {
                            0 => masked_blockers.trailing_zeros(),
                            2 => masked_blockers.trailing_zeros(),
                            4 => 63 - masked_blockers.leading_zeros(),
                            6 => 63 - masked_blockers.leading_zeros(),
                            _ => 0,
                        }
                    };
                    let blocker_ray = self.rays[first_blocker_pos as usize][i * 2];
                    self.rays[position as usize][i * 2] & !blocker_ray
                }
            };
            board |= moves;
        }
        (
            moveutil::bitboard_to_piecemoves(board, position),
            board.count_ones() as u8,
        )
    }
    fn gen_queen(&self, position: u8, blockers: u64) -> ([PieceMove; 28], u8) {
        let mut board = 0;
        for i in 0..8 {
            let masked_blockers = blockers & self.rays[position as usize][i];
            let moves = {
                if masked_blockers == 0 {
                    self.rays[position as usize][i]
                } else {
                    let first_blocker_pos = {
                        match i {
                            0 => masked_blockers.trailing_zeros(),
                            1 => masked_blockers.trailing_zeros(),
                            2 => masked_blockers.trailing_zeros(),
                            7 => masked_blockers.trailing_zeros(),
                            3 => 63 - masked_blockers.leading_zeros(),
                            4 => 63 - masked_blockers.leading_zeros(),
                            5 => 63 - masked_blockers.leading_zeros(),
                            6 => 63 - masked_blockers.leading_zeros(),
                            _ => 0,
                        }
                    };
                    let blocker_ray = self.rays[first_blocker_pos as usize][i];
                    self.rays[position as usize][i] & !blocker_ray
                }
            };
            board |= moves;
        }
        (
            moveutil::bitboard_to_piecemoves(board, position),
            board.count_ones() as u8,
        )
    }
    fn gen_king(
        &self,
        color: u8,
        position: u8,
        blockers: u64,
        castle_available: [bool; 4],
    ) -> ([PieceMove; 28], u8) {
        let mut square_moves =
            moveutil::bitboard_to_piecemoves(self.king_moves[position as usize], position);
        let mut num_moves: u8 = self.king_moves[position as usize].count_ones() as u8;
        if castle_available[(color * 2) as usize]
            && ((blockers >> (position + 1)) & 1) == 0
            && ((blockers >> (position + 2)) & 1) == 0
        {
            square_moves[num_moves as usize] = PieceMove {
                start: position,
                end: position + 2,
                special: SpecialMove::CastleKingside,
            };
            num_moves += 1;
        }
        if castle_available[(color * 2 + 1) as usize]
            && ((blockers >> (position - 1)) & 1) == 0
            && ((blockers >> (position - 2)) & 1) == 0
        {
            square_moves[num_moves as usize] = PieceMove {
                start: position,
                end: position - 2,
                special: SpecialMove::CastleQueenside,
            };
            num_moves += 1;
        }
        (square_moves, num_moves)
    }
}
