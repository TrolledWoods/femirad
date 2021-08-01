use glam::ivec2;
use board::*;

mod minmax;
mod board;

/// A function that can rate how good a board is for the current player.
pub trait ScoringFunction {
    /// The reason the board variable is mutable is so that the Ai can play around with it as a scratch-pad
    /// of sorts, after the function returns the state of the board should not have changed.
    fn score(&self, board: &mut Board) -> Score;
}

/// An `Ai` that can play moves on a board
pub trait Ai {
    /// This function should return which move it will make on a given board.
    /// The reason the board variable is mutable is so that the Ai can play around with it as a scratch-pad
    /// of sorts, after the function returns the state of the board should not have changed.
    fn pick_move(&self, board: &mut Board) -> Option<Move>;
}

#[derive(Default, Clone, Copy)]
pub struct BasicScore;

impl ScoringFunction for BasicScore {
    fn score(&self, board: &mut Board) -> Score {
        Score(
            board.score * if board.current_player == Player::A { 1 } else { -1 },
            board.moves.len() as i32,
        )
    }
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Score(i32, i32);

fn fast_board_score(board: &Board) -> i32 {
    board.score * if board.current_player == Player::A { 1 } else { -1 }
}

fn score_board(board: &mut Board, recursion: usize) -> (Option<Move>, Score) {
    if recursion == 0 {
        // println!("Found board end");
        return (None, Score(fast_board_score(board), 0));
    }

    let want_to_win = board.current_player;

    let mut moves: Vec<_> = board.get_moves().collect();

    if moves.is_empty() {
        return (
            Some(Move {
                pos: ivec2(WORLD_SIZE as i32 / 2, WORLD_SIZE as i32 / 2),
                player: board.current_player,
            }),
            Score(0, 0),
        );
    }

    // Do the temporary thing
    moves.sort_by_key(|&r#move| {
        let result = match board.do_move(r#move) {
            Ok(Some(winner)) if winner == want_to_win => i32::MAX,
            Ok(Some(_)) => i32::MIN,
            // We take the negative here because it's the opponents move(the resulting board is for the opponent).
            Ok(None) => -fast_board_score(board),
            Err(_) => panic!("Invalid!"),
        };
        board.undo_move();
        result
    });

    // Do the full scoring of the top moves
    moves
        .into_iter()
        .rev()
        .take(14)
        .map(|r#move| {
            let result = match board.do_move(r#move) {
                Ok(Some(winner)) if winner == want_to_win => Score(i32::MAX, recursion as i32),
                Ok(Some(_)) => Score(i32::MIN, recursion as i32),
                // We take the negative here because it's the opponents move.
                Ok(None) => {
                    let Score(big, small) = score_board(board, recursion - 1).1;
                    Score(-big, -small)
                }
                Err(_) => panic!("Invalid!"),
            };
            board.undo_move();
            (Some(r#move), result)
        })
        .max_by_key(|(_, v)| *v)
        .unwrap_or((None, Score(0, 0)))
}

fn main() {
    let mut board = Board::new();
    'outer: loop {
        for _ in 0..2 {
            println!();
        }

        board.print();

        let (ai_move, score) = score_board(&mut board, 5);

        println!("AI(on your side) judges this board {}", score.0);

        let mut string = String::new();
        let _ = std::io::stdin().read_line(&mut string);

        let mut failed_move = true;

        let r#move = if string.trim().is_empty() {
            ai_move
        } else {
            Move::from_string(board.current_player, &string)
        };

        if let Some(r#move) = r#move {
            println!("Did move {:?}", r#move);
            match board.do_move(r#move) {
                Ok(won) => {
                    if won.is_some() {
                        board.print();
                        println!("We have a winner!");
                        break 'outer;
                    }

                    failed_move = false;
                }
                Err(err) => {
                    println!("Move failed because {}", err);
                }
            }
        } else {
            println!("Invalid coordinate!");
        }

        if failed_move {
            println!("Press <ENTER> to try another move");
            let _ = std::io::stdin().read_line(&mut string);
            continue 'outer;
        }

        assert_eq!(board.current_player, Player::B);
        board.print();
        if let (Some(r#move), scored) = score_board(&mut board, 6) {
            println!("The computer scores this board a {}", scored.0);
            match board.do_move(r#move) {
                Ok(Some(_)) => {
                    board.print();
                    println!("Computer won!");
                    break 'outer;
                }
                Ok(None) => {
                    println!("Computer did move");
                }
                Err(error) => {
                    println!("Computer made an invalid move! {}", error);
                    break 'outer;
                }
            }
            println!("Computer did {:?}", r#move);
        }
    }
}
