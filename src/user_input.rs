use crate::Ai;
use crate::board::{Board, Move};

#[derive(Debug, Default, Clone)]
pub struct UserInputWithHelper<T>(pub String, pub T);

impl<T> Ai for UserInputWithHelper<T> where T: Ai {
    fn requires_user_output(&self) -> bool { true }

    fn name(&self) -> &str { &self.0 }

    fn pick_move(&self, board: &mut Board) -> Option<Move> {
        loop {
            println!("Enter the move you want to do(or leave blank for ai help):");

            let mut string = String::new();
            let _ = std::io::stdin().read_line(&mut string);

            if string.trim().is_empty() {
                return self.1.pick_move(board);
            } else {
                if let Some(r#move) = Move::from_string(board.current_player, &string) {
                    if board.is_move_valid(r#move) {
                        return Some(r#move);
                    } else {
                        println!("Invalid move!");
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserInput(pub String);

impl Ai for UserInput {
    fn requires_user_output(&self) -> bool { true }

    fn name(&self) -> &str { &self.0 }

    fn pick_move(&self, board: &mut Board) -> Option<Move> {
        loop {
            println!("Enter the move you want to do:");

            let mut string = String::new();
            let _ = std::io::stdin().read_line(&mut string);

            if let Some(r#move) = Move::from_string(board.current_player, &string) {
                if board.is_move_valid(r#move) {
                    return Some(r#move);
                } else {
                    println!("Invalid move!");
                }
            }
        }
    }
}
