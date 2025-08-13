#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]


use agb::{display::{object::{Object, SpriteVram}, tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER}, Priority}, fixnum::Vector2D, include_aseprite, include_background_gfx, input::{Button, ButtonController}, rng::RandomNumberGenerator};
use alloc::vec::Vec;
use alloc::vec;

use crate::graphics::sprite_init;
use crate::logic::Direction;
pub mod graphics;
pub mod logic;
extern crate alloc;

include_background_gfx!(mod background, bg => deduplicate "gfx/bg.aseprite");
include_aseprite!(mod tile_gfx, "gfx/tiles.aseprite");


#[derive(Debug, Clone)]
struct Tile {
    object: Object,             
    pos: Vector2D<i32>,
    value: u16,
    update_obj: bool, //true when object sprite needs updating
    animate: Option<Vector2D<i32>>, //Some(destination) when animation needed
    appearing: bool, //true when tile is queued to appear
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.pos.y.partial_cmp(&other.pos.y) //prioritize comparing the y value
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pos.y.cmp(&other.pos.y).then(self.pos.x.cmp(&other.pos.x)) //compare y value before x
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Tile {}

#[derive(Debug)]
struct Game {
    board: Vec<Tile>,
    sprites: Vec<SpriteVram>,
    score: u32,
}

impl Game {

    fn new() -> Self {

        let mut tiles = vec![];

        let sprites = sprite_init();

        for y in 0..4 {
            for x in 0..4 {
                tiles.push(Tile {
                    object: Object::new(sprites[0].clone()),
                    pos: (x, y).into(),
                    value: 0,
                    update_obj: false,
                    animate: None,
                    appearing: false,
                });
            }
        }

        for tile in &mut tiles {
            tile.set_pos();
        }

        Self { board: tiles, sprites: sprites, score: 0 }
    }

    fn init(mut rng: &mut RandomNumberGenerator) -> Self {

        let mut game = Self::new();

        game.spawn_tile(&mut rng);
        game.spawn_tile(&mut rng);

        game

    }

}




pub fn run(mut gba: agb::Gba) -> ! {

    let mut gfx = gba.graphics.get();
    let mut rng = RandomNumberGenerator::new();
    let mut input = ButtonController::new();

    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut bg = RegularBackground::new(Priority::P0, RegularBackgroundSize::Background32x32, TileFormat::FourBpp);
    bg.fill_with(&background::bg);

    let mut game = Game::init(&mut rng);


    loop {

        input.update();

        //TODO: improve input system
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

        game.show_tiles(&mut frame);
        bg.show(&mut frame);

        frame.commit();

    }
}

