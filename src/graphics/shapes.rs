use crate::graphics::shape::{load_shape_from_file, Shape};

#[derive(Clone)]
pub struct Shapes {
    pub player: Shape,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> anyhow::Result<Self> {
        Ok(Self {
            player: load_shape_from_file(device, "assets/shapes/player.ron")?,
        })
    }
}
