use crate::graphics::shape::{Shape, ShapeVertex};
use lyon::math::Point;
use lyon::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use serde::Deserialize;
use std::fs;

const TILE_SIZE: f32 = 16.0;

#[derive(Clone, Debug, Deserialize)]
enum ShapeOperation {
    Begin((u32, u32)),
    LineTo((u32, u32)),
    Close,
}

#[derive(Clone, Debug, Deserialize)]
struct ShapeData {
    pub grid_size: (u32, u32),
    pub operations: Vec<ShapeOperation>,
}

impl ShapeData {
    fn build_shape(&self, device: &wgpu::Device) -> anyhow::Result<Shape> {
        let mut path_builder = Path::builder();
        let grid_offset = (self.grid_size.0 as f32 * 0.5, self.grid_size.1 as f32 * 0.5);

        let create_point = |grid_position: (u32, u32)| {
            Point::new(
                (grid_position.0 as f32 - grid_offset.0) * TILE_SIZE,
                (grid_position.1 as f32 - grid_offset.1) * TILE_SIZE,
            )
        };

        for operation in self.operations.iter() {
            match operation {
                ShapeOperation::Begin(grid_position) => {
                    path_builder.begin(create_point(*grid_position));
                }
                ShapeOperation::LineTo(grid_position) => {
                    path_builder.line_to(create_point(*grid_position));
                }
                ShapeOperation::Close => {
                    path_builder.close();
                }
            }
        }

        let path = path_builder.build();

        let mut buffers = VertexBuffers::<ShapeVertex, u16>::default();
        let mut tesselator = FillTessellator::default();

        tesselator.tessellate(
            &path,
            &FillOptions::tolerance(0.1),
            &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                ShapeVertex {
                    position: vertex.position().to_array().into(),
                    ..Default::default()
                }
            }),
        )?;

        Ok(Shape::new(device, &buffers.vertices, &buffers.indices))
    }
}

pub fn load_shape_from_file(device: &wgpu::Device, path: &str) -> anyhow::Result<Shape> {
    let shape_str = fs::read_to_string(path)?;
    let shape_data = ron::from_str::<ShapeData>(&shape_str)?;
    shape_data.build_shape(device)
}
