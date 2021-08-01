use crate::Ai;
use crate::board::{Board, Move};

#[derive(Clone, Copy)]
pub struct Switch<A, B>(pub A, pub B, pub usize);

impl<A, B> Ai for Switch<A, B> where A: Ai, B: Ai {
    fn requires_user_output(&self) -> bool {
        self.1.requires_user_output() || self.0.requires_user_output()
    }

    fn name(&self) -> &str { "Switch" }

    fn pick_move(&self, board: &mut Board) -> Option<Move> {
        if board.moves >= self.2 * 2 {
            self.1.pick_move(board)
        } else {
            self.0.pick_move(board)
        }
    }
}
