use crate::graphics::load_shape_from_file;
use anchor::game::GameResult;
use anchor::graphics::shape::Shape;
use anchor::graphics::WgpuContext;

#[derive(Clone)]
pub struct Shapes {
    pub player: Shape,
}

impl Shapes {
    pub fn new(wgpu: &WgpuContext) -> GameResult<Self> {
        Ok(Self {
            player: load_shape_from_file(wgpu, "assets/shapes/player.ron")?,
        })
    }
}
