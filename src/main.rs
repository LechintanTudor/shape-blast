mod game_loop;
mod graphics;

fn main() -> anyhow::Result<()> {
    game_loop::run()
}
