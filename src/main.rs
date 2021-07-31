use std::convert::TryFrom;
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

    pub fn get_moves(&self) -> impl Iterator<Item = Move> + '_ {
        let player = self.current_player;
        (0..WORLD_SIZE as i32)
            .flat_map(move |y| {
                (0..WORLD_SIZE as i32)
                    .filter(move |&x|
                        matches!(self.get(ivec2(x, y)), Some(None))
                        && (
                            matches!(self.get(ivec2(x + 1, y)),     Some(Some(_))) ||
                            matches!(self.get(ivec2(x - 1, y)),     Some(Some(_))) ||
                            matches!(self.get(ivec2(x, y + 1)),     Some(Some(_))) ||
                            matches!(self.get(ivec2(x, y - 1)),     Some(Some(_))) ||
                            matches!(self.get(ivec2(x + 1, y + 1)), Some(Some(_))) ||
                            matches!(self.get(ivec2(x - 1, y + 1)), Some(Some(_))) ||
                            matches!(self.get(ivec2(x + 1, y - 1)), Some(Some(_))) ||
                            matches!(self.get(ivec2(x - 1, y - 1)), Some(Some(_)))
                        )
                    )
                    .map(move |x| Move::Set(ivec2(x, y), player))
            })
            .filter(move |&r#move| self.is_move_valid(r#move))
    }

    fn is_move_valid(&self, r#move: Move) -> bool {
        match r#move {
            Move::Set(pos, _) => {
                if let Some(Some(_)) | None = self.get(pos) {
                    return false;
                }

                true
            }
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

fn score_board(board: &mut Board, recursion: usize) -> (Option<Move>, i32) {
    if recursion == 0 {
        // println!("Found board end");
        return (None, 0);
    }

    let moves: Vec<_> = board.get_moves().collect();

    moves
        .into_iter()
        .map(|r#move| {
            let result = match board.do_move(r#move) {
                Ok(true) => i32::MAX,
                // We take the negative here because it's the opponents move.
                Ok(false) => -score_board(board, recursion - 1).1,
                Err(_) => panic!("Invalid!"),
            };
            board.undo_move();
            (Some(r#move), result)
        })
        .max_by_key(|(_, v)| *v)
        .unwrap_or((None, i32::MIN))
}

fn main() {
    let mut board = Board::new();
    'outer: loop {
        for _ in 0..30 {
            println!();
        }

        if board.current_player == Player::B {
            if let (Some(r#move), _) = score_board(&mut board, 5) {
                if let Ok(true) = board.do_move(r#move) {
                    board.print();
                    eprintln!("Computer won!");
                    break 'outer;
                }
            }
        }

        board.print();

        let mut string = String::new();
        let _ = std::io::stdin().read_line(&mut string);

        let mut failed_move = true;
        if let Some(r#move) = Move::from_string(board.current_player, &string) {
            match board.do_move(r#move) {
                Ok(won) => {
                    if won {
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
        }
    }
}
