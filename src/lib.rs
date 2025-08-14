#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]


use agb::{display::{object::{Object, Sprite, SpriteVram}, tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER}, Graphics, Priority}, fixnum::Vector2D, include_aseprite, include_background_gfx, input::{Button, ButtonController}, rng::RandomNumberGenerator};
use alloc::vec::Vec;
use alloc::vec;

use crate::graphics::{game_sprite_init, value_to_sprite_index, START_ANIMATION};
use crate::logic::Direction;
pub mod graphics;
pub mod logic;
extern crate alloc;

include_background_gfx!(mod background, bg => deduplicate "gfx/bg.aseprite");
include_aseprite!(mod tile_gfx, "gfx/tiles.aseprite", "gfx/tiles_menu.aseprite", "gfx/buttons.aseprite");

#[derive(Debug, Clone)]
struct Menu {
    text_one: [Object; 4],
    text_two: [Object; 4],
    button: Object,
    //sprites: Vec<SpriteVram>,
    test: bool,
    game_over: bool,
    pressed: bool,
    high_score: u32,
}

impl Menu {

    fn new() -> Self {

        Self {
            text_one: [
                Object::new(SpriteVram::from(tile_gfx::NAME.sprite(0))),
                Object::new(SpriteVram::from(tile_gfx::NAME.sprite(1))),
                Object::new(SpriteVram::from(tile_gfx::NAME.sprite(2))),
                Object::new(SpriteVram::from(tile_gfx::NAME.sprite(3))),
            ],
            text_two: [
                Object::new(SpriteVram::from(tile_gfx::PLAY.sprite(0))),
                Object::new(SpriteVram::from(tile_gfx::PLAY.sprite(1))),
                Object::new(SpriteVram::from(tile_gfx::PLAY.sprite(2))),
                Object::new(SpriteVram::from(tile_gfx::PLAY.sprite(3))),
            ],
            button: Object::new(SpriteVram::from(tile_gfx::A.sprite(0))),
            test: false,
            game_over: false,
            pressed: false,
            high_score: 0,
        }
    }


}


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
    spawn: bool,
}

impl Game {

    fn new() -> Self {

        let mut tiles = vec![];

        let sprites = game_sprite_init();

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

        Self { board: tiles, sprites: sprites, score: 0, spawn: true }
    }

    fn init_with_board(values: [u16; 16]) -> Self {

        let mut game = Self::new();

        for (i, value) in values.iter().enumerate() {

            let sprite = game.sprites[value_to_sprite_index(*value).unwrap_or(0)].clone();

            game.board[i].value = *value;
            game.board[i].object = Object::new(sprite);


        }

        game.spawn = false;

        game

    }

    fn init(mut rng: &mut RandomNumberGenerator) -> Self {

        let mut game = Self::new();

        game.spawn_tile(&mut rng);
        game.spawn_tile(&mut rng);

        game

    }

    fn play(&mut self, input: &mut ButtonController, gfx: &mut Graphics, rng: &mut RandomNumberGenerator, bg: &RegularBackground) {

        loop {


            input.update();

            //TODO: improve input system
            if input.is_just_pressed(Button::UP) {
                self.shift(Direction::Up, rng, gfx, &bg);
            } else if input.is_just_pressed(Button::DOWN) {
                self.shift(Direction::Down, rng, gfx, &bg);
            } else if input.is_just_pressed(Button::LEFT) {
                self.shift(Direction::Left, rng, gfx, &bg);
            } else if input.is_just_pressed(Button::RIGHT) {
                self.shift(Direction::Right, rng, gfx, &bg);
            }
    
            if self.spawn == false {
                if input.is_just_pressed(Button::B) {
                    self.spawn_tile(rng);
                }
            }

            if self.check_stuck() {
                break;
            }

            let mut frame = gfx.frame();
    
            bg.show(&mut frame);
            self.show_tiles(&mut frame);
            
    
            frame.commit();
        }

    }

}




pub fn run(mut gba: agb::Gba) -> ! {

    let mut gfx = gba.graphics.get();
    let mut rng = RandomNumberGenerator::new();
    let mut input = ButtonController::new();

    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut bg = RegularBackground::new(Priority::P0, RegularBackgroundSize::Background32x32, TileFormat::FourBpp);
    bg.fill_with(&background::bg);

    let mut menu = Menu::new();
    menu.set();


    loop {

        menu.game_over = false;
        menu.pressed = false;

        while !menu.pressed {

            rng.next_i32(); //call every frame to randomize rng state
            let mut frame = gfx.frame();

            bg.show(&mut frame);
            menu.show(&mut frame);

            frame.commit();


            input.update();

            //CHECK FOR PRESS

            if input.is_pressed(Button::SELECT) {
                menu.test(true);
            } else {
                menu.test(false);
            }

            if input.is_just_pressed(Button::A) {
                menu.animate(&mut gfx, &bg);
                menu.pressed = true;
            }

        }

        if menu.test {

            let tile_values = [
                8, 16, 8, 16,
                16, 8, 16, 8,
                8, 16, 8, 16,
                16, 0, 16, 8,
            ];

            let mut game = Game::init_with_board(tile_values);

            game.play(&mut input, &mut gfx, &mut rng, &bg);

        } else {

            let mut game = Game::init(&mut rng);

            game.play(&mut input, &mut gfx, &mut rng, &bg);

        }

        menu.pressed = false;
        menu.game_over = true;
        menu.set();
        menu.fade_out(&mut gfx, &bg, START_ANIMATION * 4);
        menu.fade_in(&mut gfx, &bg, START_ANIMATION);

        while !menu.pressed {

            rng.next_i32(); //call every frame to randomize rng state
            let mut frame = gfx.frame();

            bg.show(&mut frame);
            menu.show(&mut frame);

            frame.commit();


            input.update();

            if input.is_just_pressed(Button::A) {
                menu.fade_in_out(&mut gfx, &bg, START_ANIMATION / 2);
                menu.pressed = true;
            }

        }


    }
}

