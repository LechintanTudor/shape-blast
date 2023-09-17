#![allow(dead_code)]
#![allow(clippy::module_inception)]

mod game;
mod gameplay;
mod graphics;
mod input;

use crate::game::ShapeBlast;
use anchor::game::{Config, GameResult};

fn main() -> GameResult {
    anchor::run(
        ShapeBlast::new,
        Config {
            window_size: (800, 500),
            ..Default::default()
        },
    )
}
