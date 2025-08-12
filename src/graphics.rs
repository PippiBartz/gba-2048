use agb::display::object::Object;
use agb::display::tiled::RegularBackground;
use agb::display::{Graphics, GraphicsFrame};
use agb::println;
use agb::{display::object::SpriteVram, fixnum::Vector2D};
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use crate::{tile_gfx, Game, Tile};
use crate::logic::{Direction, Move};

pub const TOP_LEFT: Vector2D<i32> = Vector2D::new(56, 16);
pub const TILE_SIZE: u32 = 32;


impl Game {
    pub fn update_tiles(&mut self, frame: &mut GraphicsFrame, sprites: Vec<SpriteVram>) {

        for tile in &mut self.board.iter_mut().filter(|t| t.update_obj) {

            tile.set_pos();

            if let Some(sprite_index) = value_to_sprite(tile.value) {

                tile.object.set_sprite(sprites[sprite_index].clone());
                tile.object.show(frame);

            }

        }

    }

    pub fn animate_tiles(&mut self, gfx: &mut Graphics, bg: &RegularBackground) {

        for _i in 0..8 {


            let mut frame = gfx.frame();

            for tile in &mut self.board.iter_mut() {

                if let Some(destination) = tile.animate {

                    let destination = destination * TILE_SIZE as i32 + TOP_LEFT;

                    tile.animate(destination, &mut frame);



                } else if tile.value != 0 {
                    tile.object.show(&mut frame);
                }
                


            }

            bg.show(&mut frame);

            frame.commit();
        
        }

        for tile in self.board.iter_mut().filter(|t| t.animate.is_some()) {
            tile.animate = None;
        }

        

    }

}

impl Tile {
    pub fn set_pos(&mut self) {
        let (x, y) = (TOP_LEFT.x as u32 + self.pos.x as u32 * TILE_SIZE, TOP_LEFT.y as u32 + self.pos.y as u32 * TILE_SIZE);
        self.object.set_pos(Vector2D::new(x as i32, y as i32));
    }

    fn animate(&mut self, destination: Vector2D<i32>, frame: &mut GraphicsFrame) {

        self.object.set_pos((self.object.pos() - destination) / 3 + destination);
        self.object.show(frame);

    }


}


pub fn sprite_init() -> Vec<SpriteVram> {
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



pub fn value_to_sprite(value: u16) -> Option<usize> {

    if value == 0 {
        
        None

    } else {

        let log2_floor = 15 - value.leading_zeros();
        Some((log2_floor - 1) as usize)

    }
    
}

