use std::convert::TryFrom;
use std::io::Read;
use glam::{IVec2, ivec2};

pub const WORLD_SIZE: usize = 32;
pub const WIN_LENGTH: i32 = 5;

pub type Tile = Option<Player>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    A,
    B,
}

impl Player {
    pub fn rotate(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

pub struct Board {
    grid: [[Tile; WORLD_SIZE]; WORLD_SIZE],
    pub current_player: Player,
    moves: Vec<Move>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            grid: [[None; WORLD_SIZE]; WORLD_SIZE],
            current_player: Player::A,
            moves: Vec::new(),
        }
    }

    pub fn print(&self) {
        print!("    ");
        for i in 0..WORLD_SIZE {
            print!("{} ", char::from_digit(i as u32, 36).expect("Cannot handle a board greater than 36 in size"));
        }
        println!();

        print!("  +-");
        for _ in 0..WORLD_SIZE {
            print!("--");
        }
        println!();

        for (y, row) in self.grid.iter().enumerate() {
            print!("{} | ", char::from_digit(y as u32, 36).expect("Cannot handle a board greater than 36 in size"));

            for tile in row {
                match tile {
                    None => print!("  "),
                    Some(Player::A) => print!("X "),
                    Some(Player::B) => print!("O "),
                }
            }
            
            println!();
        }

        match self.current_player {
            Player::A => println!("X to move"),
            Player::B => println!("O to move"),
        }
    }

    pub fn get(&self, pos: IVec2) -> Option<Tile> {
        Some(*self.grid
            .get(usize::try_from(pos.y).ok()?)?
            .get(usize::try_from(pos.x).ok()?)?)
    }

    fn set(&mut self, pos: IVec2, tile: Tile) -> Option<()> {
        *self.grid
            .get_mut(usize::try_from(pos.y).ok()?)?
            .get_mut(usize::try_from(pos.x).ok()?)?
            = tile;
        Some(())
    }

    fn check_win_for_direction(&self, wanted: Player, direction: IVec2) -> bool {
        for y in 0..WORLD_SIZE as i32 - direction.y * WIN_LENGTH {
            for x in 0..WORLD_SIZE as i32 - direction.x * WIN_LENGTH {
                let mut win = true;
                for i in 0..WIN_LENGTH {
                    if let Some(Some(player)) = self.get(ivec2(x, y) + direction * i) {
                        if player != wanted {
                            win = false;
                            break;
                        }
                    } else {
                        win = false;
                        break;
                    }
                }

                if win {
                    return true;
                }
            }
        }

        false
    }

    pub fn check_win(&self, wanted: Player) -> bool {
        self.check_win_for_direction(wanted, ivec2(1, 0)) |
        self.check_win_for_direction(wanted, ivec2(1, 1)) |
        self.check_win_for_direction(wanted, ivec2(0, 1))
    }

    pub fn do_move(&mut self, r#move: Move) -> Result<bool, &'static str> {
        match r#move {
            Move::Set(pos, player) => {
                if let Some(Some(_)) = self.get(pos) {
                    return Err("Something is already at this spot");
                }

                let result = self.set(pos, Some(player));
                // If the set failed, then don't update the current player
                if result.is_none() {
                    return Err("Invalid coordinate");
                }
            }
        }

        self.moves.push(r#move);

        let won = self.check_win(self.current_player);

        self.current_player = self.current_player.rotate();

        Ok(won)
    }

    pub fn undo_move(&mut self) {
        if let Some(r#move) = self.moves.pop() {
            match r#move {
                Move::Set(pos, _) => {
                    self.set(pos, None);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Set(IVec2, Player),
}

impl Move {
    pub fn from_string(current_player: Player, string: &str) -> Option<Self> {
        let mut chars = string.trim().chars();
        let x = chars.next()?.to_digit(36)? as i32;
        let y = chars.next()?.to_digit(36)? as i32;
        let pos = ivec2(x, y);

        if chars.next().is_some() {
            return None;
        }

        Some(Self::Set(pos, current_player))
    }
}

fn main() {
    let mut board = Board::new();
    'outer: loop {
        for _ in 0..30 {
            println!();
        }

        board.print();

        let mut string = String::new();
        let _ = std::io::stdin().read_line(&mut string);

        let mut failed_move = true;
        if let Some(r#move) = Move::from_string(board.current_player, &string) {
            match board.do_move(r#move) {
                Ok(won) => {
                    if won {
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
        }
    }
}
