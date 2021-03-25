use plankton::game::Game;
use plankton::piecemove::PieceMove;
use plankton::specialmove::SpecialMove;
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;
use std::{io, thread};

fn main() {
    let mut game = Game::new();
    game.starting_game();
    println!("plankton-rs");
    let mut color = 0;
    let mut debug = false;
    for input_str in io::stdin().lock().lines() {
        let input: Vec<String> = input_str
            .unwrap()
            .trim()
            .split(' ')
            .map(|x| x.to_owned())
            .collect();
        if input[0] == "quit" {
            break;
        }
        match Some(&*input[0].to_string()) {
            Some("uci") => {
                println!("id name Plankton Engine");
                println!("id author Nosrep");
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("ucinewgame") => game.starting_game(),
            Some("position") => {
                game.starting_game();
                let mut offset = 3;
                if input.len() <= 2 {
                    color = 0;
                } else {
                    color = if input.len() % 2 == 0 { 1 } else { 0 };
                }
                if input[1] == "fen" {
                    if input.len() <= 8 {
                        color = if input[3] == "w" { 0 } else { 1 };
                    }
                    let mut fen = String::new();
                    for chunk in &input[2..] {
                        fen.push_str(&*(chunk.to_owned() + " "));
                    }
                    parse_fen(&mut game, fen);
                    offset = 9;
                }
                if debug {
                    println!("info string color {}", color);
                }
                for idx in offset..input.len() {
                    let start = input[idx].as_bytes()[0] - 97 + (input[idx].as_bytes()[1] - 49) * 8;
                    let end = input[idx].as_bytes()[2] - 97 + (input[idx].as_bytes()[3] - 49) * 8;
                    let move_color = if idx % 2 == 0 { 1 } else { 0 };
                    let piece = game.square_to_piece[start as usize];
                    let mut special = SpecialMove::None;
                    if piece == 5 {
                        if start as i8 - end as i8 == 2 {
                            special = SpecialMove::CastleQueenside;
                        } else if end as i8 - start as i8 == 2 {
                            special = SpecialMove::CastleKingside;
                        }
                    } else if piece == 0 {
                        if end < 8 || (end >= 56 && end < 64) {
                            match input[idx].as_bytes()[4] as char {
                                'n' => special = SpecialMove::KnightPromotion,
                                'b' => special = SpecialMove::BishopPromotion,
                                'r' => special = SpecialMove::RookPromotion,
                                'q' => special = SpecialMove::QueenPromotion,
                                _ => (),
                            }
                        } else {
                            if (start as i8 - end as i8).abs() != 8
                                && (start as i8 - end as i8).abs() != 16
                                && !game.square_exists[end as usize]
                            {
                                special = SpecialMove::EnPassant;
                            }
                        }
                    }
                    game.make_move(
                        move_color,
                        piece,
                        &PieceMove {
                            start,
                            end,
                            special,
                        },
                    );
                    if debug {
                        println!(
                            "info string {:?}",
                            PieceMove {
                                start,
                                end,
                                special,
                            }
                        );
                    }
                    color = move_color ^ 1;
                }
            }
            Some("go") => {
                let mut times: [i128; 2] = [-1; 2];
                let mut move_time: i128 = -1;
                let mut depth = -1;
                for idx in (1..input.len()).step_by(2) {
                    match Some(&*input[idx].to_string()) {
                        Some("wtime") => times[0] = input[idx + 1].parse().unwrap(),
                        Some("btime") => times[1] = input[idx + 1].parse().unwrap(),
                        Some("movetime") => move_time = input[idx + 1].parse().unwrap(),
                        Some("depth") => depth = input[idx + 1].parse().unwrap(),
                        _ => (),
                    }
                }
                let mut time = if move_time != -1 {
                    move_time
                } else {
                    if color == 0 {
                        times[0] / 35
                    } else {
                        times[1] / 35
                    }
                };
                time += 1000;
                if time > 15000 {
                    time = 15000;
                }
                let mut game_copy = game.clone();
                thread::spawn(move || {
                    let print_bestmove = |best_move: (PieceMove, f64)| {
                        let start_pos = (best_move.0.start % 8, best_move.0.start / 8);
                        let end_pos = (best_move.0.end % 8, best_move.0.end / 8);
                        let mut print_string = "bestmove ".to_owned();
                        print_string.push((start_pos.0 + 97) as char);
                        print_string.push_str(&*(start_pos.1 + 1).to_string());
                        print_string.push((end_pos.0 + 97) as char);
                        print_string.push_str(&*(end_pos.1 + 1).to_string());
                        match best_move.0.special {
                            SpecialMove::KnightPromotion => print_string.push('k'),
                            SpecialMove::BishopPromotion => print_string.push('b'),
                            SpecialMove::RookPromotion => print_string.push('r'),
                            SpecialMove::QueenPromotion => print_string.push('q'),
                            _ => (),
                        }
                        println!("{}", print_string)
                    };
                    if depth != -1 {
                        let start_time = Instant::now();
                        print_bestmove(
                            plankton::best_move(
                                &mut game_copy,
                                color,
                                depth as u8,
                                i128::max_value(),
                            )
                            .unwrap(),
                        );
                        if debug {
                            println!("info time {}", start_time.elapsed().as_millis());
                        }
                    } else {
                        let mut fallback = (PieceMove::default(), 0.0);
                        let start_time = Instant::now();
                        for search_depth in 1..=255 {
                            let search_time = time - start_time.elapsed().as_millis() as i128;
                            if search_time <= 0 {
                                print_bestmove(fallback);
                                break;
                            }
                            let best_move = plankton::best_move(
                                &mut game_copy,
                                color,
                                search_depth,
                                search_time,
                            );
                            match best_move {
                                Some(best_move) => fallback = best_move,
                                None => {
                                    print_bestmove(fallback);
                                    if debug {
                                        println!("info time {}", start_time.elapsed().as_millis());
                                    }
                                    break;
                                }
                            }
                        }
                    }
                });
            }
            Some("debug") => {
                debug = match Some(&*input[1].to_string()) {
                    Some("on") => true,
                    Some("off") => false,
                    _ => debug,
                }
            }
            _ => (),
        }
    }
}

fn parse_fen(game: &mut Game, fen: String) {
    game.blank_game();
    let fen_sections: Vec<&str> = fen.split_whitespace().collect();
    let mut board_rows: Vec<&str> = fen_sections[0].split("/").collect();
    board_rows.reverse();
    let mut char_to_piece: HashMap<char, u8> = HashMap::new();
    char_to_piece.insert('p', 0);
    char_to_piece.insert('n', 1);
    char_to_piece.insert('b', 2);
    char_to_piece.insert('r', 3);
    char_to_piece.insert('q', 4);
    char_to_piece.insert('k', 5);
    for (i, row) in board_rows.iter().enumerate() {
        let i = board_rows.len() - i - 1;
        let mut offset = 0;
        let mut j = 0;
        while j < 8 {
            let new_idx = j - offset;
            if (row.as_bytes()[j - offset] as char).is_digit(10) {
                j += (row.as_bytes()[new_idx] - '1' as u8) as usize;
                offset += (row.as_bytes()[new_idx] - '1' as u8) as usize;
            } else {
                let color = if ((row.as_bytes()[new_idx]) as char).is_uppercase() {
                    0
                } else {
                    1
                };
                let lower = ((row.as_bytes()[new_idx]) as char)
                    .to_lowercase()
                    .collect::<Vec<_>>()[0];
                game.create_piece(
                    color,
                    *char_to_piece.get(&lower).unwrap(),
                    (56 - 8 * i + j) as u8,
                );
            }
            j += 1;
        }
    }
    if fen_sections[2] != "-" {
        for castle_char in fen_sections[2].as_bytes() {
            let castle_char = *castle_char as char;
            match castle_char {
                'K' => game.castle_available[0] = true,
                'k' => game.castle_available[2] = true,
                'Q' => game.castle_available[1] = true,
                'q' => game.castle_available[3] = true,
                _ => (),
            }
        }
    } else {
        game.castle_available = [false; 4];
    }
    game.set_moves();
}
