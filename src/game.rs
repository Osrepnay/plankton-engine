use crate::movegen::MoveGen;
use crate::moveutil;
use crate::piecemove::PieceMove;
use crate::prevgamestate::PrevGameState;
use crate::specialmove::SpecialMove;

#[derive(Clone, PartialEq)]
pub struct Game {
    pub piece_positions: [[u64; 6]; 2],
    pub square_to_color: [u8; 64],
    pub square_to_piece: [u8; 64],
    pub square_exists: [bool; 64],
    pub square_moves: [[PieceMove; 28]; 64],
    pub square_num_moves: [u8; 64],
    pub castle_available: [bool; 4],
    move_gen: MoveGen,
}

impl Game {
    pub fn new() -> Game {
        Game {
            piece_positions: [[0; 6]; 2],
            square_to_color: [0; 64],
            square_to_piece: [0; 64],
            square_exists: [false; 64],
            square_moves: [[PieceMove::empty(); 28]; 64],
            square_num_moves: [0; 64],
            castle_available: [false; 4],
            move_gen: MoveGen::new(),
        }
    }
    pub fn game_over(&mut self, color: u8) -> bool {
        self.in_checkmate(color) || self.in_stalemate(color)
    }
    pub fn in_checkmate(&mut self, color: u8) -> bool {
        if !self.in_check(color) {
            return false;
        }
        for square in 0..64 {
            if !self.square_exists[square] || self.square_to_color[square] != color {
                continue;
            }
            let piece = self.square_to_piece[square];
            for move_idx in 0..self.square_num_moves[square] {
                let piece_move = self.square_moves[square][move_idx as usize];
                if !moveutil::legal_move(self, color, piece, &piece_move) {
                    continue;
                }
                let prev_game_state = self.make_move(color, piece, &piece_move);
                if !self.in_check(color) {
                    self.unmake_move(color, piece, &piece_move, &prev_game_state);
                    return false;
                }
                self.unmake_move(color, piece, &piece_move, &prev_game_state);
            }
        }
        true
    }
    pub fn in_stalemate(&mut self, color: u8) -> bool {
        if self.in_check(color) {
            return false;
        }
        for square in 0..self.square_moves.len() {
            if !self.square_exists[square] || self.square_to_color[square] != color {
                continue;
            }
            let piece = self.square_to_piece[square];
            for move_idx in 0..self.square_num_moves[square] {
                let piece_move = self.square_moves[square][move_idx as usize];
                if moveutil::legal_move(self, color, piece, &piece_move) {
                    return false;
                }
            }
        }
        true
    }
    pub fn in_check(&self, color: u8) -> bool {
        let opponent_color = color ^ 1;
        for square in 0..64 {
            if !self.square_exists[square] || self.square_to_color[square] != opponent_color {
                continue;
            }
            if self.piece_positions[color as usize][5]
                & moveutil::piecemoves_to_bitboard(
                    self.square_moves[square],
                    self.square_num_moves[square],
                )
                != 0
            {
                return true;
            }
        }
        false
    }
    pub fn make_move(&mut self, color: u8, piece: u8, piece_move: &PieceMove) -> PrevGameState {
        self.square_exists[piece_move.start as usize] = false;
        self.square_to_color[piece_move.start as usize] = 0;
        self.square_to_piece[piece_move.start as usize] = 0;
        let opponent_color = color ^ 1;
        let mut capture_piece = 0;
        let mut is_capture = false;
        if self.square_exists[piece_move.end as usize] {
            capture_piece = self.square_to_piece[piece_move.end as usize];
            is_capture = true;
            self.piece_positions[opponent_color as usize][capture_piece as usize] &=
                !(1u64 << piece_move.end);
        }
        self.square_exists[piece_move.end as usize] = true;
        self.square_to_color[piece_move.end as usize] = color;
        self.square_to_piece[piece_move.end as usize] = piece;
        self.piece_positions[color as usize][piece as usize] &= !(1u64 << piece_move.start);
        self.piece_positions[color as usize][piece as usize] |= 1u64 << piece_move.end;
        let prev_castle_available = self.castle_available;
        if piece == 5 {
            self.castle_available = [false; 4];
            if piece_move.special == SpecialMove::CastleKingside {
                self.piece_positions[color as usize][3] &= !(1u64 << (piece_move.end + 1));
                self.piece_positions[color as usize][3] |= 1u64 << (piece_move.end - 1);
                self.square_exists[piece_move.end as usize + 1] = false;
                self.square_to_color[piece_move.end as usize + 1] = 0;
                self.square_to_piece[piece_move.end as usize + 1] = 0;
                self.square_exists[piece_move.end as usize - 1] = true;
                self.square_to_color[piece_move.end as usize - 1] = color;
                self.square_to_piece[piece_move.end as usize - 1] = 3;
            } else if piece_move.special == SpecialMove::CastleQueenside {
                self.piece_positions[color as usize][3] &= !(1u64 << (piece_move.end - 2));
                self.piece_positions[color as usize][3] |= 1u64 << (piece_move.end + 1);
                self.square_exists[piece_move.end as usize - 2] = false;
                self.square_to_color[piece_move.end as usize - 2] = 0;
                self.square_to_piece[piece_move.end as usize - 2] = 0;
                self.square_exists[piece_move.end as usize + 1] = true;
                self.square_to_color[piece_move.end as usize + 1] = color;
                self.square_to_piece[piece_move.end as usize + 1] = 3;
            }
        } else if piece == 3 {
            match piece_move.start {
                0 => self.castle_available[1] = false,
                7 => self.castle_available[0] = false,
                56 => self.castle_available[3] = false,
                63 => self.castle_available[2] = false,
                _ => (),
            }
        } else if piece == 0 {
            match piece_move.special {
                SpecialMove::KnightPromotion => {
                    self.piece_positions[color as usize][piece as usize] &=
                        !(1u64 << piece_move.end);
                    self.piece_positions[color as usize][1] |= 1u64 << piece_move.end;
                    self.square_to_piece[piece_move.end as usize] = 1;
                }
                SpecialMove::BishopPromotion => {
                    self.piece_positions[color as usize][piece as usize] &=
                        !(1u64 << piece_move.end);
                    self.piece_positions[color as usize][2] |= 1u64 << piece_move.end;
                    self.square_to_piece[piece_move.end as usize] = 2;
                }
                SpecialMove::RookPromotion => {
                    self.piece_positions[color as usize][piece as usize] &=
                        !(1u64 << piece_move.end);
                    self.piece_positions[color as usize][3] |= 1u64 << piece_move.end;
                    self.square_to_piece[piece_move.end as usize] = 3;
                }
                SpecialMove::QueenPromotion => {
                    self.piece_positions[color as usize][piece as usize] &=
                        !(1u64 << piece_move.end);
                    self.piece_positions[color as usize][4] |= 1u64 << piece_move.end;
                    self.square_to_piece[piece_move.end as usize] = 4;
                }
                SpecialMove::EnPassant => {
                    if color == 0 {
                        if piece_move.end - piece_move.start == 7 {
                            self.piece_positions[opponent_color as usize][0] &=
                                !(1u64 << (piece_move.start - 1));
                            self.square_exists[piece_move.start as usize - 1] = false;
                            self.square_to_color[piece_move.start as usize - 1] = 0;
                            self.square_to_piece[piece_move.start as usize - 1] = 0;
                        } else {
                            self.piece_positions[opponent_color as usize][0] &=
                                !(1u64 << (piece_move.start + 1));
                            self.square_exists[piece_move.start as usize + 1] = false;
                            self.square_to_color[piece_move.start as usize + 1] = 0;
                            self.square_to_piece[piece_move.start as usize + 1] = 0;
                        }
                    } else {
                        if piece_move.start - piece_move.end == 7 {
                            self.piece_positions[opponent_color as usize][0] &=
                                !(1u64 << (piece_move.start + 1));
                            self.square_exists[piece_move.start as usize + 1] = false;
                            self.square_to_color[piece_move.start as usize + 1] = 0;
                            self.square_to_piece[piece_move.start as usize + 1] = 0;
                        } else {
                            self.piece_positions[opponent_color as usize][0] &=
                                !(1u64 << (piece_move.start - 1));
                            self.square_exists[piece_move.start as usize - 1] = false;
                            self.square_to_color[piece_move.start as usize - 1] = 0;
                            self.square_to_piece[piece_move.start as usize - 1] = 0;
                        }
                    }
                }
                _ => (),
            }
        }
        let prev_game_state = PrevGameState {
            capture_piece,
            is_capture,
            castle_available: prev_castle_available,
        };
        self.set_moves();
        prev_game_state
    }
    // WARNING: Doesn't work with en passant
    pub fn unmake_move(
        &mut self,
        color: u8,
        piece: u8,
        piece_move: &PieceMove,
        prev_game_state: &PrevGameState,
    ) {
        self.square_exists[piece_move.end as usize] = false;
        self.square_to_color[piece_move.end as usize] = 0;
        self.square_to_piece[piece_move.end as usize] = 0;
        let opponent_color = color ^ 1;
        if prev_game_state.is_capture {
            self.piece_positions[opponent_color as usize]
                [prev_game_state.capture_piece as usize] |= 1u64 << piece_move.end;
            self.square_exists[piece_move.end as usize] = true;
            self.square_to_color[piece_move.end as usize] = opponent_color;
            self.square_to_piece[piece_move.end as usize] = prev_game_state.capture_piece;
        }
        self.square_exists[piece_move.start as usize] = true;
        self.square_to_color[piece_move.start as usize] = color;
        self.square_to_piece[piece_move.start as usize] = piece;
        self.piece_positions[color as usize][piece as usize] |= 1u64 << piece_move.start;
        self.piece_positions[color as usize][piece as usize] &= !(1u64 << piece_move.end);
        self.castle_available = prev_game_state.castle_available;
        if piece == 5 {
            if piece_move.special == SpecialMove::CastleKingside {
                self.piece_positions[color as usize][3] |= 1u64 << (piece_move.end + 1);
                self.piece_positions[color as usize][3] &= !(1u64 << (piece_move.end - 1));
                self.square_exists[piece_move.end as usize - 1] = false;
                self.square_to_color[piece_move.end as usize - 1] = 0;
                self.square_to_piece[piece_move.end as usize - 1] = 0;
                self.square_exists[piece_move.end as usize + 1] = true;
                self.square_to_color[piece_move.end as usize + 1] = color;
                self.square_to_piece[piece_move.end as usize + 1] = 3;
            } else if piece_move.special == SpecialMove::CastleQueenside {
                self.piece_positions[color as usize][3] |= 1u64 << (piece_move.end - 2);
                self.piece_positions[color as usize][3] &= !(1u64 << (piece_move.end + 1));
                self.square_exists[piece_move.end as usize + 1] = false;
                self.square_to_color[piece_move.end as usize + 1] = 0;
                self.square_to_piece[piece_move.end as usize + 1] = 0;
                self.square_exists[piece_move.end as usize - 2] = true;
                self.square_to_color[piece_move.end as usize - 2] = color;
                self.square_to_piece[piece_move.end as usize - 2] = 3;
            }
        } else if piece == 0 {
            match piece_move.special {
                SpecialMove::KnightPromotion => {
                    self.piece_positions[color as usize][piece as usize] |=
                        1u64 << piece_move.start;
                    self.piece_positions[color as usize][1] &= !(1u64 << piece_move.end);
                    self.square_to_piece[piece_move.start as usize] = 0;
                }
                SpecialMove::BishopPromotion => {
                    self.piece_positions[color as usize][piece as usize] |=
                        1u64 << piece_move.start;
                    self.piece_positions[color as usize][2] &= !(1u64 << piece_move.end);
                    self.square_to_piece[piece_move.start as usize] = 0;
                }
                SpecialMove::RookPromotion => {
                    self.piece_positions[color as usize][piece as usize] |=
                        1u64 << piece_move.start;
                    self.piece_positions[color as usize][3] &= !(1u64 << piece_move.end);
                    self.square_to_piece[piece_move.start as usize] = 0;
                }
                SpecialMove::QueenPromotion => {
                    self.piece_positions[color as usize][piece as usize] |=
                        1u64 << piece_move.start;
                    self.piece_positions[color as usize][4] &= !(1u64 << piece_move.end);
                    self.square_to_piece[piece_move.start as usize] = 0;
                }
                SpecialMove::EnPassant => {
                    if color == 0 {
                        if piece_move.end - piece_move.start == 7 {
                            self.piece_positions[opponent_color as usize][0] |=
                                1u64 << (piece_move.start - 1);
                            self.square_exists[piece_move.start as usize - 1] = true;
                            self.square_to_color[piece_move.start as usize - 1] = opponent_color;
                            self.square_to_piece[piece_move.start as usize - 1] = 0;
                        } else {
                            self.piece_positions[opponent_color as usize][0] |=
                                1u64 << (piece_move.start + 1);
                            self.square_exists[piece_move.start as usize + 1] = true;
                            self.square_to_color[piece_move.start as usize + 1] = opponent_color;
                            self.square_to_piece[piece_move.start as usize + 1] = 0;
                        }
                    } else {
                        if piece_move.start - piece_move.end == 7 {
                            self.piece_positions[opponent_color as usize][0] |=
                                1u64 << (piece_move.start + 1);
                            self.square_exists[piece_move.start as usize + 1] = true;
                            self.square_to_color[piece_move.start as usize + 1] = opponent_color;
                            self.square_to_piece[piece_move.start as usize + 1] = 0;
                        } else {
                            self.piece_positions[opponent_color as usize][0] |=
                                1u64 << (piece_move.start - 1);
                            self.square_exists[piece_move.start as usize - 1] = true;
                            self.square_to_color[piece_move.start as usize - 1] = opponent_color;
                            self.square_to_piece[piece_move.start as usize - 1] = 0;
                        }
                    }
                }
                _ => (),
            }
        }
        self.set_moves();
    }
    pub fn set_moves(&mut self) {
        let mut blockers = 0;
        for color in 0..self.piece_positions.len() {
            for piece in 0..self.piece_positions[color].len() {
                blockers |= self.piece_positions[color][piece];
            }
        }
        for square in 0..self.square_moves.len() {
            if self.square_exists[square] {
                let square_move = self.move_gen.gen_move(
                    self.square_to_color[square],
                    self.square_to_piece[square],
                    square as u8,
                    blockers,
                    self.castle_available,
                );
                self.square_moves[square] = square_move.0;
                self.square_num_moves[square] = square_move.1;
            }
        }
    }
    pub fn starting_game(&mut self) {
        self.blank_game();
        for i in 0..8 {
            self.create_piece(0, 0, i + 8);
            self.create_piece(1, 0, i + 48);
        }
        self.create_piece(0, 1, 1);
        self.create_piece(0, 1, 6);
        self.create_piece(0, 2, 2);
        self.create_piece(0, 2, 5);
        self.create_piece(0, 3, 0);
        self.create_piece(0, 3, 7);
        self.create_piece(0, 4, 3);
        self.create_piece(0, 5, 4);
        self.create_piece(1, 1, 57);
        self.create_piece(1, 1, 62);
        self.create_piece(1, 2, 58);
        self.create_piece(1, 2, 61);
        self.create_piece(1, 3, 56);
        self.create_piece(1, 3, 63);
        self.create_piece(1, 4, 59);
        self.create_piece(1, 5, 60);
        self.castle_available = [true; 4];
        self.set_moves();
    }
    pub fn blank_game(&mut self) {
        self.piece_positions = [[0; 6]; 2];
        self.square_to_color = [0; 64];
        self.square_to_piece = [0; 64];
        self.square_exists = [false; 64];
        self.square_moves = [[PieceMove::empty(); 28]; 64];
        self.square_num_moves = [0; 64];
        self.castle_available = [false; 4];
        self.move_gen = MoveGen::new();
    }
    pub fn create_piece(&mut self, color: u8, piece: u8, position: u8) {
        self.piece_positions[color as usize][piece as usize] |= 1u64 << position;
        self.square_to_color[position as usize] = color;
        self.square_to_piece[position as usize] = piece;
        self.square_exists[position as usize] = true;
    }
    pub fn delete_piece(&mut self, position: u8) {
        let color = self.square_to_color[position as usize];
        let piece = self.square_to_piece[position as usize];
        self.piece_positions[color as usize][piece as usize] &= !(1u64 << position);
        self.square_to_color[position as usize] = 0;
        self.square_to_piece[position as usize] = 0;
        self.square_exists[position as usize] = false;
    }
}
