use crate::graphics::WgpuContext;
use glam::{Mat4, Vec2};
use std::ops::Deref;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub size: Vec2,
    pub anchor: Vec2,
}

impl Camera {
    pub fn ortho_matrix(&self) -> Mat4 {
        let tl_corner = self.size * (-0.5 - self.anchor);
        let br_corner = self.size * (0.5 - self.anchor);
        Mat4::orthographic_rh(tl_corner.x, br_corner.x, br_corner.y, tl_corner.y, 0.0, 1.0)
    }
}

#[derive(Debug)]
struct CameraBindGroupData {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone, Debug)]
pub struct CameraBindGroup(Arc<CameraBindGroupData>);

impl CameraBindGroup {
    fn new(buffer: wgpu::Buffer, bind_group: wgpu::BindGroup) -> Self {
        Self(Arc::new(CameraBindGroupData { buffer, bind_group }))
    }
}

impl Deref for CameraBindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.0.bind_group
    }
}

pub struct CameraManager {
    context: WgpuContext,
    projection_bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<CameraBindGroup>,
    used_bind_groups: usize,
}

impl CameraManager {
    pub fn new(context: WgpuContext) -> Self {
        let projection_bind_group_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("projection_bind_group"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        Self {
            context,
            projection_bind_group_layout,
            bind_groups: Vec::new(),
            used_bind_groups: 0,
        }
    }

    pub fn projection_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.projection_bind_group_layout
    }

    pub fn clear(&mut self) {
        self.used_bind_groups = 0;
    }

    pub fn alloc_bind_group(&mut self, projection: &Mat4) -> &CameraBindGroup {
        if self.used_bind_groups < self.bind_groups.len() {
            self.context.queue().write_buffer(
                &self.bind_groups[self.used_bind_groups].0.buffer,
                0,
                bytemuck::bytes_of(projection),
            );
        } else {
            let buffer =
                self.context
                    .device()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("projection_buffer"),
                        contents: bytemuck::bytes_of(projection),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });

            let bind_group = self
                .context
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("projection_bind_gorup"),
                    layout: &self.projection_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });

            self.bind_groups
                .push(CameraBindGroup::new(buffer, bind_group));
        }

        self.used_bind_groups += 1;
        self.bind_groups.last().unwrap()
    }
}
