#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]


use agb::{display::{object::{Object, SpriteVram}, tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER}, Priority}, fixnum::Vector2D, include_aseprite, include_background_gfx, input::{Button, ButtonController, Tri}, println, rng::RandomNumberGenerator};
use alloc::vec::Vec;
use alloc::vec;

use crate::graphics::{sprite_init, value_to_sprite};
use crate::logic::Direction;
pub mod graphics;
pub mod logic;
extern crate alloc;

include_background_gfx!(mod background, bg => deduplicate "gfx/bg.aseprite");
include_aseprite!(mod tile_gfx, "gfx/tiles.aseprite");

const SIZE: usize = 4 * 4; // 4x4 grid


#[derive(Debug, Clone)]
struct Tile {
    object: Object,
    pos: Vector2D<i32>,
    value: u16,
    update_obj: bool,
    animate: Option<Vector2D<i32>>,
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.pos.y.partial_cmp(&other.pos.y)
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pos.y.cmp(&other.pos.y).then(self.pos.x.cmp(&other.pos.x))
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Tile {}

#[derive(Debug, PartialEq)]
struct Game {
    board: Vec<Tile>,
    score: u32,
}


impl Game {
    fn new(sprite: SpriteVram) -> Self {

        let mut tiles = vec![];

        for y in 0..4 {
            for x in 0..4 {
                tiles.push(Tile {
                    object: Object::new(sprite.clone()),
                    pos: (x, y).into(),
                    value: 0,
                    update_obj: false,
                    animate: None,
                });
            }
        }

        for tile in &mut tiles {
            tile.set_pos();
        }


        Self { board: tiles, score: 0 }
    }

    fn _new_custom(sprites: Vec<SpriteVram>, tile_values: Vec<u16>) -> Self {

        let mut tiles: Vec<Tile> = vec![];

        for tile in 0..16 {

            if tile_values[tile] == 0 {
                tiles.push(Tile {
                    object: Object::new(sprites[0].clone()),
                    pos: (tile as i32 / 4, tile as i32 % 4).into(),
                    value: 0,
                    update_obj: false,
                    animate: None,
                });
            } else {
                tiles.push(Tile {
                    object: Object::new(sprites[value_to_sprite(tile_values[tile]).unwrap()].clone()),
                    pos: (tile as i32 / 4, tile as i32 % 4).into(),
                    value: tile_values[tile],
                    update_obj: false,
                    animate: None,
                })
            }
        }

        Self { board: tiles, score: 0 }


    }

    fn init(mut rng: &mut RandomNumberGenerator, sprite: SpriteVram) -> Self {

        let mut game = Self::new(sprite);

        game.spawn_tile(&mut rng);
        game.spawn_tile(&mut rng);

        game

    }

}




pub fn run(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut rng = RandomNumberGenerator::new();
    let mut frame = gfx.frame();
    let mut input = ButtonController::new();



    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut bg = RegularBackground::new(Priority::P0, RegularBackgroundSize::Background32x32, TileFormat::FourBpp);
    bg.fill_with(&background::bg);

    let sprites = sprite_init();


    let mut game = Game::init(&mut rng, sprites[0].clone());

    game.update_tiles(&mut frame, sprites.clone());
    

    loop {

        input.update();

        

        if input.is_just_pressed(Button::UP) {
            game.shift(Direction::Up, &mut rng, &mut gfx, &bg);
        } else if input.is_just_pressed(Button::DOWN) {
            game.shift(Direction::Down, &mut rng, &mut gfx, &bg);
        } else if input.is_just_pressed(Button::LEFT) {
            game.shift(Direction::Left,  &mut rng, &mut gfx, &bg);
        } else if input.is_just_pressed(Button::RIGHT) {
            game.shift(Direction::Right, &mut rng, &mut gfx, &bg);
        }




        let mut frame = gfx.frame();

        game.update_tiles(&mut frame, sprites.clone());
        bg.show(&mut frame);



        frame.commit();

    }
}

