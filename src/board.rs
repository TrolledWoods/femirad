use std::convert::TryFrom;
use crate::ScoringFunction;
use glam::{IVec2, ivec2};

pub const WORLD_SIZE: usize = 16;
pub const WIN_LENGTH: i32 = 5;

#[derive(Default, Clone)]
pub struct Board {
    grid: [[Tile; WORLD_SIZE]; WORLD_SIZE],
    pub current_player: Player,
    pub moves: Vec<Move>,
    pub score: i32,
    score_stack: Vec<i32>,
    pub won: Option<Player>,

    pub player_a_one_left: i32,
    pub player_b_one_left: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub pos: IVec2,
    pub player: Player,
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

        Some(Self {
            pos,
            player: current_player,
        })
    }
}

pub type Tile = Option<Player>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    A,
    B,
}

impl Default for Player {
    fn default() -> Self {
        Self::A
    }
}

impl Player {
    pub fn rotate(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn print(&mut self) {
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
                    None if x % 5 == 4 && y % 5 == 4 => print!(": "), // print!("{}{}", char::from_digit(x as u32, 36).unwrap(), char::from_digit(y as u32, 36).unwrap()),
                    None if y % 5 == 4 => print!(". "),
                    None if x % 5 == 4 => print!(": "),
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

        /*println!("The score of the board is {}", self.score);
        println!("The \"better\" score of the board is {}", crate::BetterBasicScore.score(self).0);
        println!("A one left is {}", self.player_a_one_left);
        println!("B one left is {}", self.player_b_one_left);*/
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
                    .map(move |x| Move { pos: ivec2(x, y), player })
            })
            .filter(move |&r#move| self.is_move_valid(r#move))
    }

    pub fn is_move_valid(&self, r#move: Move) -> bool {
        !matches!(self.get(r#move.pos), Some(Some(_)) | None)
    }

    fn pos_directional_score(&self, pos: IVec2, direction: IVec2) -> (i32, i32, i32, Option<Player>) {
        let mut score = 0_i32;

        let mut player_a_one_left = 0_i32;
        let mut player_b_one_left = 0_i32;

        let mut player_a = 0_i32;
        let mut player_b = 0_i32;

        let mut winner = None;

        for i in -(WIN_LENGTH as i32)..0 {
            match self.get(pos + direction * i) {
                Some(Some(Player::A)) => player_a += 1,
                Some(Some(Player::B)) => player_b += 1,
                Some(None) => {},
                None => {},
            }
        }

        for i in 0 .. WIN_LENGTH as i32 {
            match self.get(pos + direction * i) {
                Some(Some(Player::A)) => player_a += 1,
                Some(Some(Player::B)) => player_b += 1,
                Some(None) => {},
                None => {
                    continue;
                },
            }

            match self.get(pos + direction * (i - (WIN_LENGTH as i32))) {
                Some(Some(Player::A)) => player_a -= 1,
                Some(Some(Player::B)) => player_b -= 1,
                Some(None) => {},
                None => {
                    continue;
                },
            }

            if player_a == 0 {
                if player_b == WIN_LENGTH {
                    winner = Some(Player::B);
                } else {
                    if player_b == WIN_LENGTH - 1 {
                        player_b_one_left += 1;
                    }

                    score -= player_b.pow(2);
                }
            } else if player_b == 0 {
                if player_a == WIN_LENGTH {
                    winner = Some(Player::A);
                } else {
                    if player_a == WIN_LENGTH - 1 {
                        player_a_one_left += 1;
                    }

                    score += player_a.pow(2);
                }
            }
        }

        (score, player_a_one_left, player_b_one_left, winner)
    }

    pub fn score_for_position(&self, pos: IVec2) -> (i32, i32, i32, Option<Player>) {
        let (score0, player_a_one_left0, player_b_one_left0, winner0) = self.pos_directional_score(pos, ivec2(0, 1));
        let (score1, player_a_one_left1, player_b_one_left1, winner1) = self.pos_directional_score(pos, ivec2(1, 1));
        let (score2, player_a_one_left2, player_b_one_left2, winner2) = self.pos_directional_score(pos, ivec2(-1, 1));
        let (score3, player_a_one_left3, player_b_one_left3, winner3) = self.pos_directional_score(pos, ivec2(1, 0));

        (
            score0.saturating_add(score1).saturating_add(score2).saturating_add(score3),
            player_a_one_left0 + player_a_one_left1 + player_a_one_left2 + player_a_one_left3,
            player_b_one_left0 + player_b_one_left1 + player_b_one_left2 + player_b_one_left3,
            winner0.or(winner1).or(winner2).or(winner3),
        )
    }

    pub fn do_move(&mut self, r#move: Move) -> Result<Option<Player>, &'static str> {
        if self.won.is_some() {
            return Err("Cannot do a move when someone has won");
        }

        let Move { pos, player } = r#move;

        if let Some(Some(_)) = self.get(pos) {
            return Err("Something is already at this spot");
        }

        let old_score = self.score;
        let old_player_a_one_left = self.player_a_one_left;
        let old_player_b_one_left = self.player_b_one_left;
        let (score, a_one_left, b_one_left, _) = self.score_for_position(pos);
        self.score -= score;
        self.player_a_one_left -= a_one_left;
        self.player_b_one_left -= b_one_left;
        let result = self.set(pos, Some(player));
        let (score, a_one_left, b_one_left, winner) = self.score_for_position(pos);
        self.score += score;
        self.player_a_one_left += a_one_left;
        self.player_b_one_left += b_one_left;

        // If the set failed, then don't update the current player
        if result.is_none() {
            self.score = old_score;
            self.player_a_one_left = old_player_a_one_left;
            self.player_b_one_left = old_player_b_one_left;
            return Err("Invalid coordinate");
        }

        self.moves.push(r#move);
        self.score_stack.push(old_score);

        self.current_player = self.current_player.rotate();

        if let Some(winner) = winner {
            self.won = Some(winner);
        }

        Ok(winner)
    }

    pub fn undo_move(&mut self) {
        if let Some(r#move) = self.moves.pop() {
            self.won = None;

            let Move { pos, .. } = r#move;

            let (score, a_one_left, b_one_left, _) = self.score_for_position(pos);
            self.score -= score;
            self.player_a_one_left -= a_one_left;
            self.player_b_one_left -= b_one_left;

            self.set(pos, None);

            let (score, a_one_left, b_one_left, _) = self.score_for_position(pos);
            self.score += score;
            self.player_a_one_left += a_one_left;
            self.player_b_one_left += b_one_left;

            assert_eq!(Some(self.score), self.score_stack.pop());

            self.current_player = self.current_player.rotate();
        }
    }
}

