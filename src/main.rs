use glam::ivec2;
use board::*;
use user_input::UserInput;
use minmax::MinMax;

mod user_input;
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
    fn requires_user_output(&self) -> bool { false }
    fn name(&self) -> &str;

    /// This function should return which move it will make on a given board.
    /// The reason the board variable is mutable is so that the Ai can play around with it as a scratch-pad
    /// of sorts, after the function returns the state of the board should not have changed.
    ///
    /// The move returned has to be valid.
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

pub fn run_match(player_a: impl Ai, player_b: impl Ai, print_debugging: bool) -> Player {
    let mut board = Board::new();

    let require_user_output = player_a.requires_user_output() || player_b.requires_user_output();

    loop {
        if print_debugging || require_user_output {
            println!("{}s move.", player_a.name());
            board.print();
        }

        match player_a.pick_move(&mut board) {
            Some(r#move) => {
                let _ = board.do_move(r#move);
                if print_debugging {
                    println!("{} did {:?}", player_a.name(), r#move);
                }
            }
            None => {
                if print_debugging || require_user_output {
                    println!("{} forfeit!", player_a.name());
                }
                return Player::B;
            },
        }

        if let Some(won) = board.won {
            if print_debugging || require_user_output {
                board.print();
                println!("{} won!", player_a.name());
            }
            return won;
        }

        if print_debugging || require_user_output {
            println!("{}s move.", player_b.name());
            board.print();
        }

        match player_b.pick_move(&mut board) {
            Some(r#move) => {
                let _ = board.do_move(r#move);
                if print_debugging {
                    println!("{} did {:?}", player_b.name(), r#move);
                }
            }
            None => {
                if print_debugging || require_user_output {
                    println!("{} forfeit!", player_b.name());
                }
                return Player::A;
            },
        }

        if let Some(won) = board.won {
            if print_debugging ||require_user_output {
                board.print();
                println!("{} won!", player_b.name());
            }
            return won;
        }
    }
}

fn main() {
    run_match(
        UserInput("Trolled".to_string()),
        MinMax::new(BasicScore, 5),
        false,
    );
}
