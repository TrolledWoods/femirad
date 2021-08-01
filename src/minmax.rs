use crate::{Ai, ScoringFunction, Score, WORLD_SIZE};
use crate::board::{Board, Move};
use glam::ivec2;

#[derive(Default, Clone, Copy)]
pub struct MinMax<T> {
    pub score: T,
    pub depth: u32,
}

impl<T> MinMax<T> {
    pub fn new(score: T, depth: u32) -> Self {
        Self {
            score,
            depth,
        }
    }
}

impl<T> ScoringFunction for MinMax<T> where T: ScoringFunction {
    fn score(&self, board: &mut Board) -> Score {
        self.do_minmax(board, self.depth).1
    }
}

impl<T> Ai for MinMax<T> where T: ScoringFunction {
    fn name(&self) -> &str {
        "Min max"
    }

    fn pick_move(&self, board: &mut Board) -> Option<Move> {
        self.do_minmax(board, self.depth).0
    }
}

impl<T> MinMax<T> where T: ScoringFunction {
    fn do_minmax(&self, board: &mut Board, recursion: u32) -> (Option<Move>, Score) {
        if recursion == 0 {
            // println!("Found board end");
            return (None, self.score.score(board));
        }

        let want_to_win = board.current_player;

        let temp_moves: Vec<Move> = board.get_moves().collect();
        let mut moves: Vec<(Move, i32)> = temp_moves
            .into_iter()
            .map(|r#move| {
                let result = match board.do_move(r#move) {
                    Ok(Some(winner)) if winner == want_to_win => i32::MAX,
                    Ok(Some(_)) => i32::MIN,
                    // We take the negative here because it's the opponents move(the resulting board is for the opponent).
                    Ok(None) => -self.score.score(board).0,
                    Err(_) => panic!("Invalid!"),
                };
                board.undo_move();
                (r#move, result)
            })
            .collect();

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
        moves.sort_unstable_by(|(_, a), (_, b)| b.cmp(a));

        if let Some(&(r#move, i32::MAX)) = moves.get(0) {
            // The best move is a guaranteed win, we can "shortcircuit"
            return (Some(r#move), Score(i32::MAX, 0));
        }

        if let Some((_, i32::MIN)) = moves.get(0) {
            // @Robustness: We should make a check here, I don't think this case should ever trigger
        }

        // Do the full scoring of the top moves
        moves
            .into_iter()
            .take(14)
            .map(|(r#move, _)| {
                let result = match board.do_move(r#move) {
                    Ok(Some(winner)) if winner == want_to_win => Score(i32::MAX, recursion as i32),
                    Ok(Some(_)) => Score(i32::MIN, recursion as i32),
                    // We take the negative here because it's the opponents move.
                    Ok(None) => {
                        let Score(big, small) = self.do_minmax(board, recursion - 1).1;
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
}
