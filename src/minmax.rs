use crate::{Ai, ScoringFunction, Score, WORLD_SIZE, ScoreThing};
use crate::board::{Board, Move};
use glam::ivec2;
use rayon::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct MinMax<T, Q> {
    pub score: T,
    pub filter_score: Q,
    pub depth: u32,
    pub culling: usize,
}

impl<T, Q> MinMax<T, Q> {
    pub fn new(score: T, filter_score: Q, depth: u32, culling: usize) -> Self {
        Self {
            score,
            filter_score,
            depth,
            culling,
        }
    }
}

impl<T, Q> ScoringFunction for MinMax<T, Q> where T: ScoringFunction + Send + Sync, Q: ScoringFunction + Send + Sync {
    fn score(&self, board: &mut Board) -> Score {
        self.do_minmax(board, self.depth).1
    }
}

impl<T, Q> Ai for MinMax<T, Q> where T: ScoringFunction + Send + Sync, Q: ScoringFunction + Send + Sync {
    fn name(&self) -> &str {
        "Min max"
    }

    fn pick_move(&self, board: &mut Board) -> Option<Move> {
        let (r#move, score) = self.do_minmax(board, self.depth);
        println!("Board score from minmax: {}", score.0);
        r#move
    }
}

impl<T, Q> MinMax<T, Q> where T: ScoringFunction + Send + Sync, Q: ScoringFunction + Send + Sync {
    fn do_minmax(&self, board: &mut Board, recursion: u32) -> (Option<Move>, Score) {
        if recursion == 0 {
            // println!("Found board end");
            return (None, self.score.score(board));
        }

        let want_to_win = board.current_player;

        let temp_moves: Vec<Move> = board.get_moves().collect();
        let mut moves: Vec<(Move, ScoreThing)> = temp_moves
            .into_par_iter()
            .map_with(*board, |board, r#move| {
                let handle = board.do_reversible_move(r#move);
                let result = match handle.board.won {
                    Some(winner) if winner == want_to_win => ScoreThing::Max,
                    Some(_) => ScoreThing::Min,
                    // We take the negative here because it's the opponents move(the resulting board is for the opponent).
                    None => self.filter_score.score(handle.board).0.invert(),
                };
                drop(handle);
                (r#move, result)
            })
            .collect();

        if moves.is_empty() {
            return (
                Some(Move {
                    pos: ivec2(WORLD_SIZE as i32 / 2, WORLD_SIZE as i32 / 2),
                    player: board.current_player,
                }),
                Score(ScoreThing::Score(0), 0),
            );
        }

        // Do the temporary thing
        moves.sort_unstable_by(|(_, a), (_, b)| b.cmp(a));

        if let Some(&(r#move, ScoreThing::Max)) = moves.get(0) {
            // The best move is a guaranteed win, we can "shortcircuit"
            return (Some(r#move), Score(ScoreThing::Max, 0));
        }

        if let Some((_, ScoreThing::Min)) = moves.get(0) {
            // @Robustness: We should make a check here, I don't think this case should ever trigger
        }

        // Do the full scoring of the top moves
        moves[..self.culling]
            .par_iter()
            .map_with(*board, |board, &(r#move, _)| {
                let handle = board.do_reversible_move(r#move);
                let result = match handle.board.won {
                    Some(winner) if winner == want_to_win => Score(ScoreThing::Max, recursion as i32),
                    Some(_) => Score(ScoreThing::Min, recursion as i32),
                    // We take the negative here because it's the opponents move.
                    None => {
                        let Score(big, small) = self.do_minmax(handle.board, recursion - 1).1;
                        Score(big.invert(), -small)
                    }
                };
                drop(handle);
                (Some(r#move), result)
            })
            .max_by_key(|(_, v)| *v)
            .unwrap_or((None, Score(ScoreThing::Score(0), 0)))
    }
}
