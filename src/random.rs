use crate::{Ai, WORLD_SIZE};
use crate::board::{Board, Move};
use glam::ivec2;

pub struct Random;

impl Ai for Random {
    fn name(&self) -> &str { "Random" }

    fn pick_move(&self, board: &mut Board) -> Option<Move> {
        let hashset: std::collections::HashSet<_> = board.get_moves().collect();
        hashset.into_iter().next().or(Some(Move {
            pos: ivec2(WORLD_SIZE as i32 / 2, WORLD_SIZE as i32 / 2),
            player: board.current_player,
        }))
    }
}
