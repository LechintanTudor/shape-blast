#![allow(dead_code)]
#![allow(clippy::module_inception)]

use crate::game::Game;

mod game;
mod gameplay;
mod graphics;
mod input;

fn main() -> anyhow::Result<()> {
    Game::run()
}
