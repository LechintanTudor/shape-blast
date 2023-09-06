use crate::graphics::shape::{Shape, ShapeVertex};
use crate::graphics::{vertex_attr_array, CameraManager, Color, RendererStatus, WgpuContext};
use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Vec2, Vec4};
use std::mem;
use wgpu::util::DeviceExt;

#[repr(C)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ShapeInstance {
    scale_rotation_col_0: Vec2,
    scale_rotation_col_1: Vec2,
    translation: Vec2,
    anchor_offset: Vec2,
    linear_color: Vec4,
}

impl Default for ShapeInstance {
    fn default() -> Self {
        Self {
            scale_rotation_col_0: Vec2::new(1.0, 0.0),
            scale_rotation_col_1: Vec2::new(0.0, 1.0),
            translation: Vec2::ZERO,
            anchor_offset: Vec2::ZERO,
            linear_color: Vec4::ONE,
        }
    }
}

#[derive(Clone, Debug)]
struct ShapeBatch {
    shape: Shape,
    start_instance: u32,
    instance_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ShapeParams {
    pub transform: Affine2,
    pub anchor_offset: Vec2,
    pub color: Color,
}

impl Default for ShapeParams {
    fn default() -> Self {
        Self {
            transform: Affine2::IDENTITY,
            anchor_offset: Vec2::ZERO,
            color: Color::WHITE,
        }
    }
}

#[derive(Debug)]
pub struct ShapeRenderer {
    context: WgpuContext,
    pipeline: wgpu::RenderPipeline,
    pipeline_layout: wgpu::PipelineLayout,
    shader_module: wgpu::ShaderModule,
    batches: Vec<ShapeBatch>,
    instances: Vec<ShapeInstance>,
    status: RendererStatus,
    instance_buffer: Option<wgpu::Buffer>,
}

impl ShapeRenderer {
    pub fn new(
        context: WgpuContext,
        camera_manager: &CameraManager,
        texture_format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> Self {
        let device = context.device();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shape_pipeline_layout"),
            bind_group_layouts: &[camera_manager.projection_bind_group_layout()],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shape_vertex"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shape_shader.wgsl").into()),
        });

        let pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader_module,
            texture_format,
            sample_count,
        );

        Self {
            context,
            pipeline,
            pipeline_layout,
            shader_module,
            batches: Vec::new(),
            instances: Vec::new(),
            status: RendererStatus::Empty,
            instance_buffer: None,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader_module: &wgpu::ShaderModule,
        texture_format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shape_pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<ShapeVertex>() as _,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attr_array!(ShapeVertex {
                            0 => position: Float32x2,
                            1 => linear_color: Float32x4,
                        }),
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<ShapeInstance>() as _,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &vertex_attr_array!(ShapeInstance {
                            2 => scale_rotation_col_0: Float32x2,
                            3 => scale_rotation_col_1: Float32x2,
                            4 => translation: Float32x2,
                            5 => anchor_offset: Float32x2,
                            6 => linear_color: Float32x4,
                        }),
                    },
                ],
                module: shader_module,
                entry_point: "vs_main",
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    pub fn begin(&mut self) {
        self.batches.clear();
        self.instances.clear();
        self.status = RendererStatus::Empty;
    }

    pub fn add(&mut self, shape: &Shape, params: ShapeParams) {
        match self
            .batches
            .last_mut()
            .filter(|batch| &batch.shape == shape)
        {
            Some(batch) => batch.instance_count += 1,
            None => {
                self.batches.push(ShapeBatch {
                    shape: shape.clone(),
                    start_instance: self.instances.len() as _,
                    instance_count: 1,
                });
            }
        }

        self.instances.push(ShapeInstance {
            scale_rotation_col_0: params.transform.matrix2.col(0),
            scale_rotation_col_1: params.transform.matrix2.col(1),
            translation: params.transform.translation,
            anchor_offset: params.anchor_offset,
            linear_color: params.color.as_linear_vec4(),
        });

        self.status = RendererStatus::NeedsUpload;
    }

    pub fn end(&mut self) {
        if self.status != RendererStatus::NeedsUpload {
            return;
        }

        let instances_size =
            (self.instances.len() * mem::size_of::<ShapeInstance>()) as wgpu::BufferAddress;

        match self.instance_buffer.as_ref() {
            Some(instance_buffer) if instances_size <= instance_buffer.size() => {
                self.context.queue().write_buffer(
                    instance_buffer,
                    0,
                    bytemuck::cast_slice(&self.instances),
                );
            }
            _ => {
                self.instance_buffer = Some(self.context.device().create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("shape_instance_buffer"),
                        contents: bytemuck::cast_slice(&self.instances),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    },
                ));
            }
        }

        self.status = RendererStatus::Ready;
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        match self.status {
            RendererStatus::Empty => return,
            RendererStatus::NeedsUpload => panic!("must call 'end' before drawing"),
            _ => (),
        }

        let Some(instance_buffer) = self.instance_buffer.as_ref() else {
            return;
        };

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(1, instance_buffer.slice(..));

        for batch in self.batches.iter() {
            pass.set_vertex_buffer(0, batch.shape.vertex_buffer().slice(..));

            pass.set_index_buffer(
                batch.shape.index_buffer().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            let instance_range =
                batch.start_instance..(batch.start_instance + batch.instance_count);

            pass.draw_indexed(0..batch.shape.index_count(), 0, instance_range)
        }
    }
}
