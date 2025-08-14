use agb::display::object::{GraphicsMode, Sprite};
use agb::display::tiled::RegularBackground;
use agb::display::{Graphics, GraphicsFrame, Priority};
use agb::fixnum::{num, Num};
use agb::println;
use agb::{display::object::SpriteVram, fixnum::Vector2D};
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::{tile_gfx, Game, Menu, Tile};

pub const TOP_LEFT: Vector2D<i32> = Vector2D::new(56, 16);
pub const TILE_SIZE: u32 = 32;

pub const ANIMATION_TIME: i32 = 8;
pub const START_ANIMATION: i32 = ANIMATION_TIME * 4;

impl Menu {

    pub fn set(&mut self) {
        self.set_pos();
        self.set_objs();
    }

    fn set_pos(&mut self) {
        for (i, letter) in self.text.iter_mut().enumerate() {

            let (x, y) = (TOP_LEFT.x as u32 + TILE_SIZE * i as u32, TOP_LEFT.y as u32 + TILE_SIZE);
            letter.set_pos(Vector2D::new(x as i32, y as i32));

        }
        let (x, y) = (TOP_LEFT.x as u32 + TILE_SIZE, TOP_LEFT.y as u32 + TILE_SIZE * 2);
        self.button.set_pos(Vector2D::new(x as i32, y as i32));
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        for letter in self.text.iter_mut() {
            letter.show(frame);
        }
        self.button.show(frame);
    }

    fn set_objs(&mut self) {

        if self.test {
            for (i, letter) in self.text.iter_mut().enumerate() {
                letter.set_sprite(SpriteVram::from(tile_gfx::TEST.sprite(i)));
            }
        } else {
            for (i, letter) in self.text.iter_mut().enumerate() {
                letter.set_sprite(SpriteVram::from(tile_gfx::PLAY.sprite(i)));
            }
        }

    }

    pub fn test(&mut self, test: bool) {
        if test != self.test {
            self.test = test;
            self.set_objs();
        }
    }

    pub fn animate(&mut self, gfx: &mut Graphics, bg: &RegularBackground) {

        self.animate_start(gfx, bg);
        self.animate_end(gfx, bg);

    }

    fn animate_end(&mut self, gfx: &mut Graphics, bg: &RegularBackground) {

        let blend_amts: [Num<u8, 4>; 5] = [
            num!(1.0),
            num!(0.75),
            num!(0.50),
            num!(0.25),
            num!(0.0),
        ];

        let mut blend_lvl = 0;

        for anim_frame in 0..ANIMATION_TIME {

            let mut frame = gfx.frame();

            let bg_id = bg.show(&mut frame);

            frame.blend().darken(blend_amts[blend_lvl]).enable_background(bg_id).enable_object();

            if anim_frame % 2 == 0 && blend_lvl < 4 {
                blend_lvl += 1;
            }

            frame.commit();

        }

    }

    fn animate_start(&mut self, gfx: &mut Graphics, bg: &RegularBackground) {

        for letter in self.text.iter_mut() {
            letter.set_graphics_mode(GraphicsMode::AlphaBlending);
        }

        self.button.set_sprite(SpriteVram::from(tile_gfx::A.sprite(1)));
        self.button.set_graphics_mode(GraphicsMode::AlphaBlending);

        let blend_amts: [Num<u8, 4>; 5] = [
            num!(0.0),
            num!(0.25),
            num!(0.50),
            num!(0.75),
            num!(1.0),
        ];

        let mut blend_lvl = 0;
        

        for anim_frame in 0..START_ANIMATION {

            let mut frame = gfx.frame();

            let bg_id = bg.show(&mut frame);

            for (i, letter) in self.text.iter_mut().enumerate() {

                if (anim_frame % 16) / 4 == i as i32 {
                    self.in_motion[i] = true;
                }

                if self.in_motion[i] == true && !is_above(letter.pos()) {
                    letter.set_pos(letter.pos() + Vector2D::new(0, -anim_frame));
                }

            }

            if anim_frame >= START_ANIMATION / 4 {

                frame.blend().darken(blend_amts[blend_lvl]).enable_background(bg_id).enable_object();

                if anim_frame < START_ANIMATION / 2 {
                    if anim_frame % 2 == 0 && blend_lvl < 4 {
                        blend_lvl += 1;
                    }
                }
                
            }


            self.show(&mut frame);

            frame.commit();



        }

    }


}


impl Game {

    fn update_tile_objs(&mut self) {

        //iterates through every tile with the update_obj flag set
        for tile in &mut self.board.iter_mut().filter(|t| t.update_obj) {

            tile.set_obj(self.sprites.clone());

        }

    } 


    pub fn show_tiles(&mut self, frame: &mut GraphicsFrame) {

        for tile in &mut self.board.iter_mut() {

            if tile.update_obj {
                tile.set_obj(self.sprites.clone());
            }
            
            tile.show(frame);

        }

    }

    pub fn animate_move_tiles(&mut self, gfx: &mut Graphics, bg: &RegularBackground) {

        for _i in 0..ANIMATION_TIME {

            let mut frame = gfx.frame();

            for tile in &mut self.board.iter_mut() {

                if let Some(destination) = tile.animate {

                    tile.animate_move(destination, &mut frame);

                } else if !tile.appearing {

                    tile.show(&mut frame); //so that all tiles are displayed during animation frames

                }
                
            }

            bg.show(&mut frame);

            frame.commit();
        
        }

        //reset animation and appearing flag once completed
        for tile in self.board.iter_mut().filter(|t| t.animate.is_some() || t.appearing) {
            tile.animate = None;
            tile.appearing = false;
        }

    }

}

impl Tile {

    //reset all objects to default position
    pub fn set_pos(&mut self) {
        let (x, y) = (TOP_LEFT.x as u32 + self.pos.x as u32 * TILE_SIZE, TOP_LEFT.y as u32 + self.pos.y as u32 * TILE_SIZE);
        self.object.set_pos(Vector2D::new(x as i32, y as i32));
    }

    fn animate_move(&mut self, destination: Vector2D<i32>, frame: &mut GraphicsFrame) {

        let og_position_adjusted = self.pos * TILE_SIZE as i32 + TOP_LEFT;
        let destination_adjusted = destination * TILE_SIZE as i32 + TOP_LEFT;

        self.object.set_pos(self.object.pos() + (destination_adjusted - og_position_adjusted) / (ANIMATION_TIME + 1));
        self.object.set_priority(Priority::P0);
        self.object.show(frame);

    }

    fn set_obj(&mut self, sprites: Vec<SpriteVram>) {

        if let Some(sprite_index) = value_to_sprite_index(self.value) {
            self.object.set_sprite(sprites[sprite_index].clone());
        }
        self.update_obj = false;

    }

    fn show(&mut self, frame: &mut GraphicsFrame) {
        self.set_pos();

        if self.value != 0 {
            self.object.show(frame);
        }
    }

}


pub fn game_sprite_init() -> Vec<SpriteVram> {
    vec![
        SpriteVram::from(tile_gfx::TWO.sprite(0)),
        SpriteVram::from(tile_gfx::FOUR.sprite(0)),
        SpriteVram::from(tile_gfx::EIGHT.sprite(0)),
        SpriteVram::from(tile_gfx::SIXTEEN.sprite(0)),
        SpriteVram::from(tile_gfx::THIRTYTWO.sprite(0)),
        SpriteVram::from(tile_gfx::SIXTYFOUR.sprite(0)),
        SpriteVram::from(tile_gfx::ONETWENTYEIGHT.sprite(0)),
        SpriteVram::from(tile_gfx::TWOFIFTYSIX.sprite(0)),
        SpriteVram::from(tile_gfx::FIVETWELVE.sprite(0)),
        SpriteVram::from(tile_gfx::TENTWENTYFOUR.sprite(0)),
        SpriteVram::from(tile_gfx::TWENTYFOURTYEIGHT.sprite(0)),
    ]
}



pub fn value_to_sprite_index(value: u16) -> Option<usize> {

    if value == 0 {
        
        None

    } else {

        let log2_floor = 15 - value.leading_zeros();
        Some((log2_floor - 1) as usize)

    }
    
}

fn is_above(pos: Vector2D<i32>) -> bool {

    pos.y > 128 && pos.y < 192

}

