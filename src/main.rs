use std::cmp::Ord;
use std::fmt::{self, Write};
use glam::ivec2;
use board::*;
use user_input::{UserInput, UserInputWithHelper};
use minmax::MinMax;
use random::Random;
use switch::Switch;

mod switch;
mod random;
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
            if board.current_player == Player::A {
                if board.won == Some(Player::B) {
                    ScoreThing::Min
                } else {
                    ScoreThing::Score(board.score)
                }
            } else {
                if board.won == Some(Player::A) {
                    ScoreThing::Min
                } else {
                    ScoreThing::Score(board.score).invert()
                }
            },
            -(board.moves as i32),
        )
    }
}

#[derive(Default, Clone, Copy)]
pub struct BetterBasicScore;

impl ScoringFunction for BetterBasicScore {
    fn score(&self, board: &mut Board) -> Score {
        let score = match board.current_player {
            Player::A => {
                if board.player_a_one_left >= 1 {
                    ScoreThing::Max
                } else if false &&  board.player_b_one_left >= 2 {
                    ScoreThing::Min
                } else if board.won == Some(Player::A) {
                    ScoreThing::Max
                } else if board.won == Some(Player::B) {
                    ScoreThing::Min
                } else {
                    ScoreThing::Score(board.score)
                }
            }
            Player::B => {
                if board.player_b_one_left >= 1 {
                    ScoreThing::Max
                } else if false && board.player_a_one_left >= 2 {
                    ScoreThing::Min
                } else if board.won == Some(Player::B) {
                    ScoreThing::Max
                } else if board.won == Some(Player::A) {
                    ScoreThing::Min
                } else {
                    ScoreThing::Score(board.score).invert()
                }
            }
        };

        Score(
            score,
            -(board.moves as i32),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreThing {
    Max,
    Score(i32),
    Min,
}

impl fmt::Display for ScoreThing {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Min => write!(fmt, "min"),
            Self::Score(v) => write!(fmt, "{}", v),
            Self::Max => write!(fmt, "max"),
        }
    }
}

impl std::cmp::PartialOrd for ScoreThing {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoreThing {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        match (self, other) {
            (Self::Min,      Self::Min     ) => Equal,
            (Self::Min,      _             ) => Less,
            (Self::Score(_), Self::Min     ) => Greater,
            (Self::Score(a), Self::Score(b)) => a.cmp(b),
            (Self::Score(_), Self::Max     ) => Less,
            (Self::Max,      Self::Max     ) => Equal,
            (Self::Max,      _             ) => Greater,
        }
    }
}

impl ScoreThing {
    /// Adds a score to this value
    pub fn add(self, score: i32) -> Self {
        match self {
            Self::Max => Self::Max,
            Self::Score(v) => Self::Score(v + score),
            Self::Min => Self::Min,
        }
    }

    pub fn invert(self) -> Self {
        match self {
            Self::Max => Self::Min,
            Self::Score(v) => Self::Score(-v),
            Self::Min => Self::Max,
        }
    }
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Score(ScoreThing, i32);

impl Score {
    /// Adds a score to this value
    pub fn add(self, score: i32) -> Self {
        Self(self.0.add(score), self.1)
    }

    pub fn invert(self) -> Self {
        Self(self.0.invert(), self.1)
    }
}

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
            if print_debugging || require_user_output {
                board.print();
                println!("{} won!", player_b.name());
            }
            return won;
        }
    }
}

fn main() {
    run_match(
        // Switch(Random, MinMax::new(BetterBasicScore, BetterBasicScore, 5, 10), 0),
        UserInput("Trolled".to_string()),
        Switch(Random, MinMax::new(BetterBasicScore, BetterBasicScore, 6, 10), 0),
        true,
    );

        // Random,
        // Random,
        // UserInput("Trolled".to_string()),
        // UserInputWithHelper("Trolled2".to_string(), MinMax::new(BetterBasicScore, 2)),
}
