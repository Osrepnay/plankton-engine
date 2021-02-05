use crate::game::Game;
use crate::piecemove::PieceMove;
use crate::specialmove::SpecialMove;

pub fn legal_move(game: &mut Game, color: u8, piece: u8, piece_move: &PieceMove) -> bool {
    if game.square_exists[piece_move.end as usize]
        && game.square_to_color[piece_move.end as usize] == color
    {
        return false;
    }
    if piece_move.special == SpecialMove::CastleKingside {
        if game.square_exists[piece_move.start as usize + 1]
            || game.square_exists[piece_move.start as usize + 2]
        {
            return false;
        }
        let mid_castle_move = PieceMove {
            start: piece_move.start,
            end: piece_move.start + 1,
            special: SpecialMove::None,
        };
        let prev_game_state = game.make_move(color, piece, &mid_castle_move);
        if prev_game_state.is_capture || game.in_check(color) {
            game.unmake_move(color, piece, &mid_castle_move, &prev_game_state);
            return false;
        }
        game.unmake_move(color, piece, &mid_castle_move, &prev_game_state);
    } else if piece_move.special == SpecialMove::CastleQueenside {
        if game.square_exists[piece_move.start as usize - 1]
            || game.square_exists[piece_move.start as usize - 2]
            || game.square_exists[piece_move.start as usize - 3]
        {
            return false;
        }
        let mid_castle_move = PieceMove {
            start: piece_move.start,
            end: piece_move.start - 1,
            special: SpecialMove::None,
        };
        let prev_game_state = game.make_move(color, piece, &mid_castle_move);
        if prev_game_state.is_capture || game.in_check(color) {
            game.unmake_move(color, piece, &mid_castle_move, &prev_game_state);
            return false;
        }
        game.unmake_move(color, piece, &mid_castle_move, &prev_game_state);
    }
    let prev_move_state = game.make_move(color, piece, piece_move);
    if game.in_check(color) {
        game.unmake_move(color, piece, piece_move, &prev_move_state);
        return false;
    }
    game.unmake_move(color, piece, piece_move, &prev_move_state);
    true
}

pub fn bitboard_to_piecemoves(board: u64, start: u8) -> [PieceMove; 28] {
    let mut square_moves = [PieceMove::empty(); 28];
    let mut num_moves = 0;
    for i in 0..64 {
        if ((board >> i) & 1) != 0 {
            square_moves[num_moves] = PieceMove {
                start,
                end: i,
                special: SpecialMove::None,
            };
            num_moves += 1;
        }
    }
    square_moves
}

pub fn piecemoves_to_bitboard(piece_moves: [PieceMove; 28], num_moves: u8) -> u64 {
    let mut bitboard = 0;
    for move_idx in 0..num_moves {
        bitboard |= 1 << piece_moves[move_idx as usize].end;
    }
    bitboard
}
