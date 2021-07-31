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
    score: i32,
    score_stack: Vec<i32>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            grid: [[None; WORLD_SIZE]; WORLD_SIZE],
            current_player: Player::A,
            moves: Vec::new(),
            score: 0,
            score_stack: Vec::new(),
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

            for (x, tile) in row.iter().enumerate() {
                match tile {
                    None if x % 5 == 4 && y % 5 == 4 => print!("{}{}", char::from_digit(x as u32, 36).unwrap(), char::from_digit(y as u32, 36).unwrap()),
                    None if y % 5 == 4 => print!(".."),
                    None if x % 5 == 4 => print!(": "),
                    None => print!(". "),
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

        println!("The score of the board is {}", self.score);
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
                !matches!(self.get(pos), Some(Some(_)) | None)
            }
        }
    }

    fn pos_directional_score(&self, pos: IVec2, direction: IVec2) -> i32 {
        let mut score = 0_i32;

        let mut player_a = 0_i32;
        let mut player_b = 0_i32;

        let mut pos = pos + direction * -(WIN_LENGTH as i32);
        for i in -(WIN_LENGTH as i32) .. WIN_LENGTH as i32 {
            match self.get(pos) {
                Some(Some(Player::A)) => player_a += 1,
                Some(Some(Player::B)) => player_b += 1,
                Some(None) => {},
                None => {},
            }

            if i >= 0 {
                match self.get(pos - direction * WIN_LENGTH) {
                    Some(Some(Player::A)) => player_a -= 1,
                    Some(Some(Player::B)) => player_b -= 1,
                    Some(None) => {},
                    None => {
                        pos += direction;
                        continue;
                    },
                }

                if player_a == 0 {
                    if player_b == WIN_LENGTH {
                        score = -10000;
                    } else {
                        score = score.saturating_sub(player_b.pow(2));
                    }
                } else if player_b == 0 {
                    if player_b == WIN_LENGTH {
                        score = 10000;
                    } else {
                        score = score.saturating_add(player_a.pow(2));
                    }
                }
            }

            pos += direction;

        }

        score
    }

    pub fn score_for_position(&self, pos: IVec2) -> i32 {
        self.pos_directional_score(pos, ivec2(0, 1))
            .saturating_add(self.pos_directional_score(pos, ivec2(1, 0)))
            .saturating_add(self.pos_directional_score(pos, ivec2(1, 1)))
            .saturating_add(self.pos_directional_score(pos, ivec2(-1, 1)))
    }

    fn score_for_direction(&self, positions: impl Iterator<Item = IVec2>, direction: IVec2) -> i32 {
        let mut score = 0_i32;

        // Essentially we loop through all places where a win could occur.
        // If there's nothing there, it could be a win in the future, but no worry, we give it a score of 0 since it's not active at the moment.
        // If there is a single player there, then that player could use that place to win, so we give a score of however many tiles that player has there, cubed, since having more makes it exponentially more dangerous.
        // If both players have tiles there, then neither of them could complete the win, so it's also a score of 0.
        for mut pos in positions {
            let mut player_a = 0_i32;
            let mut player_b = 0_i32;
            
            loop {
                match self.get(pos) {
                    Some(Some(Player::A)) => player_a += 1,
                    Some(Some(Player::B)) => player_b += 1,
                    Some(None) => {},
                    None => break,
                }

                match self.get(pos - direction * WIN_LENGTH) {
                    Some(Some(Player::A)) => player_a -= 1,
                    Some(Some(Player::B)) => player_b -= 1,
                    Some(None) => {},
                    None => {
                        pos += direction;
                        continue;
                    },
                }

                if player_a == 0 {
                    if player_b == WIN_LENGTH {
                        score = -10000;
                    } else {
                        score = score.saturating_sub(player_b.pow(2));
                    }
                } else if player_b == 0 {
                    if player_b == WIN_LENGTH {
                        score = 10000;
                    } else {
                        score = score.saturating_add(player_a.pow(2));
                    }
                }

                pos += direction;
            }
        }

        score
    }

    pub fn score(&self) -> i32 {
        self.score_for_direction(
            (0..WORLD_SIZE as i32).map(|v| ivec2(0, v)),
            ivec2(1, 0),
        ).saturating_add(self.score_for_direction(
            (0..WORLD_SIZE as i32).map(|v| ivec2(0, v))
            .chain((0..WORLD_SIZE as i32).map(|v| ivec2(v, 0))),
            ivec2(1, 1),
        ))
        .saturating_add(self.score_for_direction(
            (0..WORLD_SIZE as i32).map(|v| ivec2(0, v)),
            ivec2(0, 1),
        ))
        .saturating_add(self.score_for_direction(
            (0..WORLD_SIZE as i32).map(|v| ivec2(WORLD_SIZE as i32 - 1, v))
            .chain((0..WORLD_SIZE as i32).map(|v| ivec2(v, 0))),
            ivec2(-1, 1),
        ))
    }

    fn check_win_for_direction(&self, wanted: Player, direction: IVec2) -> bool {
        for y in 0..WORLD_SIZE as i32 {
            for x in 0..WORLD_SIZE as i32 {
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
        self.check_win_for_direction(wanted, ivec2(1, 0))
        || self.check_win_for_direction(wanted, ivec2(1, 1))
        || self.check_win_for_direction(wanted, ivec2(0, 1))
        || self.check_win_for_direction(wanted, ivec2(-1, 1))
    }

    pub fn do_move(&mut self, r#move: Move) -> Result<bool, &'static str> {
        match r#move {
            Move::Set(pos, player) => {
                if let Some(Some(_)) = self.get(pos) {
                    return Err("Something is already at this spot");
                }

                let old_score = self.score;
                self.score -= self.score_for_position(pos);
                let result = self.set(pos, Some(player));
                self.score += self.score_for_position(pos);
                // If the set failed, then don't update the current player
                if result.is_none() {
                    self.score = old_score;
                    return Err("Invalid coordinate");
                }

                self.score_stack.push(old_score);
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
                    self.score -= self.score_for_position(pos);
                    self.set(pos, None);
                    self.score += self.score_for_position(pos);
                }
            }

            assert_eq!(Some(self.score), self.score_stack.pop());

            self.current_player = self.current_player.rotate();
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

fn fast_board_score(board: &Board) -> i32 {
    board.score * if board.current_player == Player::A { 1 } else { -1 }
}

fn score_board(board: &mut Board, recursion: usize) -> (Option<Move>, i32) {
    if recursion == 0 {
        // println!("Found board end");
        return (None, fast_board_score(board));
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
        for _ in 0..2 {
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
        if let (Some(r#move), _) = score_board(&mut board, 4) {
            match board.do_move(r#move) {
                Ok(true) => {
                    board.print();
                    println!("Computer won!");
                    break 'outer;
                }
                Ok(false) => {
                    println!("Computer did move");
                }
                Err(error) => {
                    println!("Computer made an invalid move! {}", error);
                    break 'outer;
                }
            }
        }
    }
}
