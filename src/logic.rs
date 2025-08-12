use core::iter;

use agb::display::tiled::RegularBackground;
use agb::display::{Graphics, GraphicsFrame};
use agb::println;
use agb::{display::object::SpriteVram, fixnum::Vector2D, rng::RandomNumberGenerator};
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Move {
    start: usize,
    end: usize,
    upgrade: bool,
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

use crate::{Game, Tile};

pub fn scale_rng(num: i32, min: impl Into<u32>, max: impl Into<u32>) -> u32 {

    let num_u32 = num.abs() as u32;

    let max = max.into();
    let min = min.into(); 

    num_u32 % ((max - min) + 1) + min

}



impl Game {

    pub fn spawn_tile(&mut self, rng: &mut RandomNumberGenerator) {

        let mut blanks = vec![];

        for (i, tile) in self.board.iter().enumerate() {

            if tile.value == 0 {

                blanks.push((i, tile.clone()));

            }

        }

        let rand_index = scale_rng(rng.next_i32(), 0_u32, (blanks.len() - 1) as u32) as usize;
        let rand_value = scale_rng(rng.next_i32(), 0_u32, 3_u32);

        let rand_blank = &mut blanks[rand_index];

        self.board[rand_blank.0].update_obj = true;
        self.board[rand_blank.0].value = if rand_value == 0 {4} else {2};

    }

    pub fn shift(&mut self, dir: Direction, rng: &mut RandomNumberGenerator, gfx: &mut Graphics, bg: &RegularBackground) {

        if self.shift_tiles(dir) {
            self.animate_tiles(gfx, bg);
            self.spawn_tile(rng);
        }

    }

    fn shift_tiles(&mut self, dir: Direction) -> bool {

        let moves = Move::get(&self.board, dir);

        if !moves.is_empty() {

            for m in moves {

                if m.upgrade {
                    self.board[m.end].value *= 2;
                } else {
                    self.board[m.end].value = self.board[m.start].value;
                }
    
                self.board[m.start].value = 0;
    
                self.board[m.start].update_obj = true;
                self.board[m.start].animate = Some(m.to_vec2d().1);
                self.board[m.end].update_obj = true;

            }
            true
        } else {
            false
        }
    }

}


impl Move {

    fn to_vec2d(&self) -> (Vector2D<i32>, Vector2D<i32>) {

        let start = self.start as i32;
        let end = self.end as i32;

        (Vector2D::new(start % 4, start / 4), Vector2D::new(end % 4, end / 4))

    }

    fn new(positions: (usize, usize,)) -> Self {
        Move { start: positions.0, end: positions.1, upgrade: false }
    }

    fn upgrade(&mut self) -> Self {
        self.upgrade = true;
        self.clone()
    }


    fn validate(&mut self, board: &Vec<Tile>) -> Option<Self> {

        if board[self.end].value == 0 {
            Some(*self)
        } else if board[self.end].value == board[self.start].value {
            self.upgrade();
            Some(*self)
        } else {
            None
        }

    }

    fn move_up(board: &Vec<Tile>, index: usize) -> Option<Self> {

        let mut mv = None;

        for distance in 1..=3 {
            if index >= distance * 4 {
                if let Some(validated) = Self::new((index, index - distance * 4)).validate(board) {
                    mv = Some(validated);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        mv

    }

    fn move_down(board: &Vec<Tile>, index: usize) -> Option<Self> {

        let mut mv = None;

        for distance in 1..=3 {
            if index < 16 - distance * 4 {
                if let Some(validated) = Self::new((index, index + distance * 4)).validate(board) {
                    mv = Some(validated);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        mv

    }

    fn move_left(board: &Vec<Tile>, index: usize) -> Option<Self> {

        let mut mv = None;

        for distance in 1..=3 {
            if index >= distance && index - distance >= (index / 4) * 4 {
                if let Some(validated) = Self::new((index, index - distance)).validate(board) {
                    mv = Some(validated);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        mv

    }

    fn move_right(board: &Vec<Tile>, index: usize) -> Option<Self> {

        let mut mv = None;

        for distance in 1..=3 {
            if index + distance < ((index / 4) + 1) * 4 {
                if let Some(validated) = Self::new((index, index + distance)).validate(board) {
                    mv = Some(validated);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        mv

    }

    

    fn get(board: &Vec<Tile>, dir: Direction) -> Vec<Self> {

        let mut moves = vec![];

        let mut board_future = board.clone();

        let mut board_sorted = board.clone();
        board_sorted.sort();

        match dir {
            Direction::Up | Direction::Left => {
                
                for (index, _tile) in board_sorted.iter().enumerate().filter(|t| t.1.value != 0) {

                    if dir == Direction::Up {
                        if let Some(mv) = Self::move_up(&board_future, index) { 
                            moves.push(mv);
                            board_future = simulate(board_future, mv); 
                        }
                    } else {
                        if let Some(mv) = Self::move_left(&board_future, index) {
                            moves.push(mv);
                            board_future = simulate(board_future, mv);
                        }
                    }

                }
            },

            Direction::Down | Direction::Right => {
                board_sorted.reverse();
                for (index, _tile) in board_sorted.iter().enumerate().filter(|t| t.1.value != 0) {

                    let index = 15 - index;

                    if dir == Direction::Down {
                        if let Some(mv) = Self::move_down(&board_future, index) {
                            moves.push(mv);
                            board_future = simulate(board_future, mv);
                        }
                    } else {
                        if let Some(mv) = Self::move_right(&board_future, index) {
                            moves.push(mv);
                            board_future = simulate(board_future, mv);
                        }
                    }

                }
            }
        }

        moves
        

    }

}

fn simulate(mut board: Vec<Tile>, mv: Move) -> Vec<Tile> {
    
    if mv.upgrade {
        board[mv.end].value = board[mv.start].value * 2;
    } else {
        board[mv.end].value = board[mv.start].value;
    }
    board[mv.start].value = 0;

    board

}



