use crate::game::Game;
use crate::piecemove::PieceMove;
use num_cpus;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

pub mod game;
pub mod magics;
pub mod movegen;
pub mod moveutil;
pub mod piecemove;
pub mod prevgamestate;
pub mod pstables;
pub mod specialmove;

const PIECE_SCORES: [f64; 6] = [1.0, 3.0, 3.25, 5.0, 9.0, 10000.0];

pub fn best_move(
    game: &mut Game,
    color: u8,
    depth: u8,
    search_time: i128,
) -> Option<(PieceMove, f64)> {
    let start_time = Instant::now();
    let mut best_move = (
        PieceMove::default(),
        if color == 0 {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        },
    );
    let mut moves = Vec::new();
    for square in 0..64 {
        if !game.square_exists[square] || game.square_to_color[square] != color {
            continue;
        }
        let piece = game.square_to_piece[square];
        for move_idx in 0..game.square_moves[square].len() {
            if start_time.elapsed().as_millis() as i128 > search_time {
                return None;
            }
            let piece_move = game.square_moves[square][move_idx as usize];
            if !moveutil::legal_move(game, color, piece, &piece_move) {
                continue;
            }
            moves.push(piece_move);
        }
    }
    let cores = num_cpus::get();
    let mut grouped_moves = vec![Vec::new(); cores];
    for (i, piece_move) in moves.into_iter().enumerate() {
        let group = i % cores;
        grouped_moves[group].push(piece_move);
    }
    let (tx, rx) = mpsc::channel();
    for move_group in grouped_moves {
        let mut game = game.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            for piece_move in move_group {
                let piece = game.square_to_piece[piece_move.start as usize];
                let prev_game_state = game.make_move(color, piece, &piece_move);
                let new_time = search_time - start_time.elapsed().as_millis() as i128 as i128;
                let score = {
                    if color == 0 {
                        min(&mut game, best_move.1, f64::INFINITY, depth - 1, new_time)
                    } else {
                        max(
                            &mut game,
                            f64::NEG_INFINITY,
                            best_move.1,
                            depth - 1,
                            new_time,
                        )
                    }
                };
                game.unmake_move(color, piece, &piece_move, &prev_game_state);
                match score {
                    Some(score) => {
                        if color == 0 {
                            if score > best_move.1 {
                                best_move = (piece_move, score);
                            }
                        } else {
                            if score < best_move.1 {
                                best_move = (piece_move, score);
                            }
                        }
                    }
                    None => tx.send(None).expect("Failed to send result of search"),
                }
            }
            tx.send(Some(best_move))
                .expect("Failed to send result of search");
        });
    }
    for _ in 0..cores {
        let potential_best_move = rx.recv().expect("Failed to read from thread receiver.");
        match potential_best_move {
            Some(potential_best_move) => {
                if color == 0 {
                    if potential_best_move.1 > best_move.1 {
                        best_move = potential_best_move;
                    }
                } else {
                    if potential_best_move.1 < best_move.1 {
                        best_move = potential_best_move;
                    }
                }
            }
            None => return None,
        }
    }
    Some(best_move)
}

pub fn max(
    game: &mut Game,
    mut alpha: f64,
    beta: f64,
    depth: u8,
    search_time: i128,
) -> Option<f64> {
    let start_time = Instant::now();
    if game.game_over(0) {
        return Some(eval(game, 0));
    }
    if depth <= 0 {
        return q_max(
            game,
            alpha,
            beta,
            search_time - start_time.elapsed().as_millis() as i128,
        );
    }
    for square in 0..64 {
        if !game.square_exists[square] || game.square_to_color[square] != 0 {
            continue;
        }
        let piece = game.square_to_piece[square];
        for move_idx in 0..game.square_moves[square].len() {
            if start_time.elapsed().as_millis() as i128 > search_time {
                return None;
            }
            let piece_move = game.square_moves[square][move_idx as usize];
            if !moveutil::legal_move(game, 0, piece, &piece_move) {
                continue;
            }
            let prev_game_state = game.make_move(0, piece, &piece_move);
            let score = min(
                game,
                alpha,
                beta,
                depth - 1,
                search_time - start_time.elapsed().as_millis() as i128,
            );
            game.unmake_move(0, piece, &piece_move, &prev_game_state);
            match score {
                Some(score) => {
                    if score >= beta {
                        return Some(beta);
                    }
                    if score > alpha {
                        alpha = score;
                    }
                }
                None => return None,
            }
        }
    }
    Some(alpha)
}
pub fn min(
    game: &mut Game,
    alpha: f64,
    mut beta: f64,
    depth: u8,
    search_time: i128,
) -> Option<f64> {
    let start_time = Instant::now();
    if game.game_over(1) {
        return Some(eval(game, 1));
    }
    if depth <= 0 {
        return q_min(
            game,
            alpha,
            beta,
            search_time - start_time.elapsed().as_millis() as i128,
        );
    }
    for square in 0..64 {
        if !game.square_exists[square] || game.square_to_color[square] != 1 {
            continue;
        }
        let piece = game.square_to_piece[square];
        for move_idx in 0..game.square_moves[square].len() {
            if start_time.elapsed().as_millis() as i128 > search_time {
                return None;
            }
            let piece_move = game.square_moves[square][move_idx as usize];
            if !moveutil::legal_move(game, 1, piece, &piece_move) {
                continue;
            }
            let prev_game_state = game.make_move(1, piece, &piece_move);
            let score = max(
                game,
                alpha,
                beta,
                depth - 1,
                search_time - start_time.elapsed().as_millis() as i128,
            );
            game.unmake_move(1, piece, &piece_move, &prev_game_state);
            match score {
                Some(score) => {
                    if score <= alpha {
                        return Some(alpha);
                    }
                    if score < beta {
                        beta = score;
                    }
                }
                None => return None,
            }
        }
    }
    Some(beta)
}
pub fn q_max(game: &mut Game, mut alpha: f64, beta: f64, search_time: i128) -> Option<f64> {
    let start_time = Instant::now();
    let stand_pat = eval(game, 0);
    if game.game_over(0) {
        return Some(stand_pat);
    }
    if stand_pat >= beta {
        return Some(beta);
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    for square in 0..64 {
        if !game.square_exists[square] || game.square_to_color[square] != 0 {
            continue;
        }
        let piece = game.square_to_piece[square];
        for move_idx in 0..game.square_moves[square].len() {
            if start_time.elapsed().as_millis() as i128 > search_time {
                return None;
            }
            let piece_move = game.square_moves[square][move_idx as usize];
            if !game.square_exists[piece_move.end as usize] {
                continue;
            }
            if !moveutil::legal_move(game, 0, piece, &piece_move) {
                continue;
            }
            if see(game, &piece_move) < 0.0 {
                continue;
            }
            let prev_game_state = game.make_move(0, piece, &piece_move);
            let score = q_min(
                game,
                alpha,
                beta,
                search_time - start_time.elapsed().as_millis() as i128,
            );
            game.unmake_move(0, piece, &piece_move, &prev_game_state);
            match score {
                Some(score) => {
                    if score >= beta {
                        return Some(beta);
                    }
                    if score > alpha {
                        alpha = score;
                    }
                }
                None => return None,
            }
        }
    }
    Some(alpha)
}
pub fn q_min(game: &mut Game, alpha: f64, mut beta: f64, search_time: i128) -> Option<f64> {
    let start_time = Instant::now();
    let stand_pat = eval(game, 1);
    if game.game_over(1) {
        return Some(stand_pat);
    }
    if stand_pat <= alpha {
        return Some(alpha);
    }
    if stand_pat < beta {
        beta = stand_pat;
    }
    for square in 0..64 {
        if !game.square_exists[square] || game.square_to_color[square] != 1 {
            continue;
        }
        let piece = game.square_to_piece[square];
        for move_idx in 0..game.square_moves[square].len() {
            if start_time.elapsed().as_millis() as i128 > search_time {
                return None;
            }
            let piece_move = game.square_moves[square][move_idx as usize];
            if !game.square_exists[piece_move.end as usize] {
                continue;
            }
            if !moveutil::legal_move(game, 1, piece, &piece_move) {
                continue;
            }
            if see(game, &piece_move) > 0.0 {
                continue;
            }
            let prev_game_state = game.make_move(1, piece, &piece_move);
            let score = q_max(
                game,
                alpha,
                beta,
                search_time - start_time.elapsed().as_millis() as i128,
            );
            game.unmake_move(1, piece, &piece_move, &prev_game_state);
            match score {
                Some(score) => {
                    if score <= alpha {
                        return Some(alpha);
                    }
                    if score < beta {
                        beta = score;
                    }
                }
                None => return None,
            }
        }
    }
    Some(beta)
}
pub fn see(game: &mut Game, piece_move: &PieceMove) -> f64 {
    let color = game.square_to_color[piece_move.start as usize];
    let piece = game.square_to_piece[piece_move.start as usize];
    let mut score = PIECE_SCORES[game.square_to_piece[piece_move.end as usize] as usize]
        * (-(color as f64) * 2.0 + 1.0);
    let prev_game_state = game.make_move(color, piece, &piece_move);
    let mut lowest_attacker_square = -1;
    let mut lowest_attacker_move = PieceMove::default();
    for square in 0..64 {
        if !game.square_exists[square] || game.square_to_color[square] == color {
            continue;
        }
        let mut capture_idx = 0;
        let mut found = false;
        for move_idx in 0..game.square_moves[square].len() {
            if game.square_moves[square][move_idx as usize].start == square as u8
                && game.square_moves[square][move_idx as usize].end == piece_move.end
            {
                capture_idx = move_idx;
                found = true;
                break;
            }
        }
        if !found {
            continue;
        }
        let capture_move = game.square_moves[square][capture_idx as usize];
        let square_color = game.square_to_color[square];
        let square_piece = game.square_to_piece[square];
        if !moveutil::legal_move(game, square_color, square_piece, &capture_move) {
            continue;
        }
        if lowest_attacker_square == -1
            || square_piece < game.square_to_piece[lowest_attacker_square as usize]
        {
            lowest_attacker_square = square as i32;
            lowest_attacker_move = capture_move;
            if square_piece == 0 {
                break;
            }
        }
    }
    if lowest_attacker_square != -1 {
        score += see(game, &lowest_attacker_move);
    }
    game.unmake_move(color, piece, piece_move, &prev_game_state);
    return score;
}
pub fn eval(game: &mut Game, color: u8) -> f64 {
    if game.in_stalemate(color) {
        return 0 as f64;
    }
    if game.in_checkmate(0) {
        return -10000 as f64;
    }
    if game.in_checkmate(1) {
        return 10000 as f64;
    }
    let mut score = 0.0;
    for piece in 0..(game.piece_positions[0].len() - 1) {
        let wscore = game.piece_positions[0][piece].count_ones() as f64 * PIECE_SCORES[piece];
        let bscore = game.piece_positions[1][piece].count_ones() as f64 * PIECE_SCORES[piece];
        score += wscore;
        score -= bscore;
        for square in 0..64 {
            // TODO implement unsymmetrical tables
            if game.square_exists[square] {
                if game.square_to_color[square] == 0 {
                    score += pstables::PIECE_SQUARE_TABLES[piece][square] as f64 / 100.0;
                } else {
                    score -= pstables::PIECE_SQUARE_TABLES[piece][63 - square] as f64 / 100.0;
                }
            }
        }
    }
    return score;
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use crate::piecemove::PieceMove;
    use crate::specialmove::SpecialMove;

    #[test]
    fn test_checkmates() {
        //rook back-rank
        let mut game = Game::new();
        game.create_piece(1, 5, 0);
        game.create_piece(0, 5, 16);
        game.create_piece(0, 3, 15);
        game.set_moves();
        assert_eq!(
            crate::best_move(&mut game, 0, 1, i128::max_value()).unwrap(),
            (
                PieceMove {
                    start: 15,
                    end: 7,
                    special: SpecialMove::None,
                },
                10000.0
            )
        );
    }
    #[test]
    fn test_stalemates() {
        let mut game = Game::new();
        game.create_piece(1, 5, 0);
        game.create_piece(1, 2, 10);
        game.create_piece(0, 4, 11);
        game.create_piece(0, 5, 63);
        game.set_moves();
        let best_move = crate::best_move(&mut game, 0, 1, i128::max_value()).unwrap();
        assert_ne!((best_move.0.start, best_move.0.end), (11, 10));
    }
    #[test]
    fn test_material_gain() {
        let mut game = Game::new();
        game.create_piece(1, 5, 0);
        game.create_piece(1, 4, 16);
        game.create_piece(0, 5, 63);
        game.create_piece(0, 1, 4);
        game.set_moves();
        let best_move = crate::best_move(&mut game, 0, 3, i128::max_value()).unwrap();
        assert_eq!((best_move.0.start, best_move.0.end), (4, 10));
    }
    #[test]
    fn test_check_functions() {
        let mut game = Game::new();
        game.create_piece(1, 5, 0);
        game.create_piece(0, 5, 16);
        game.create_piece(0, 3, 7);
        game.set_moves();
        assert!(game.in_check(1));
        assert!(game.in_checkmate(1));
    }
    #[test]
    fn test_movegen() {
        //TODO Add more tests
        let mut game = Game::new();
        game.create_piece(0, 0, 8);
        game.set_moves();
        assert_eq!(
            game.square_moves[8][0..2],
            [
                PieceMove {
                    start: 8,
                    end: 16,
                    special: SpecialMove::None,
                },
                PieceMove {
                    start: 8,
                    end: 24,
                    special: SpecialMove::None,
                }
            ][..]
        )
    }
    #[test]
    fn test_see() {
        let mut game = Game::new();
        game.create_piece(0, 5, 0);
        game.create_piece(1, 5, 2);
        game.create_piece(1, 0, 55);
        game.create_piece(0, 3, 46);
        game.create_piece(0, 0, 37);
        game.set_moves();
        assert!(
            (crate::see(
                &mut game,
                &PieceMove {
                    start: 55,
                    end: 46,
                    special: SpecialMove::None,
                }
            ) - -4.0)
                .abs()
                < 0.5
        );
    }
}
