use crate::graphics::shape::{Shape, ShapeRenderer, ShapeVertex};
use crate::graphics::{Camera, CameraManager, GraphicsContext};
use glam::{UVec2, Vec2};
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WNIDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 500.0;

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    let window = WindowBuilder::new()
        .with_title("Shape Blast")
        .with_inner_size(LogicalSize::new(WNIDOW_WIDTH, WINDOW_HEIGHT))
        .with_min_inner_size(LogicalSize::new(WNIDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0))
        .build(&event_loop)?;

    let mut graphics = unsafe { GraphicsContext::new(&window)? };
    let mut camera_manager = CameraManager::new(graphics.context.clone());
    let mut shape_renderer = ShapeRenderer::new(
        graphics.context.clone(),
        &camera_manager,
        graphics.surface_config.format,
        1,
    );

    let (window_width, window_height): (u32, u32) = window.inner_size().into();

    let mut camera = Camera {
        size: Vec2::new(window_width as _, window_height as _),
        anchor: Vec2::ZERO,
    };

    let shape = Shape::new(
        graphics.context.device(),
        &[
            ShapeVertex {
                position: Vec2::new(-100.0, -100.0),
                ..Default::default()
            },
            ShapeVertex {
                position: Vec2::new(-100.0, 100.0),
                ..Default::default()
            },
            ShapeVertex {
                position: Vec2::new(100.0, 100.0),
                ..Default::default()
            },
            ShapeVertex {
                position: Vec2::new(100.0, -100.0),
                ..Default::default()
            },
        ],
        &[0, 1, 3, 3, 1, 2],
    );

    event_loop.run(|event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Poll) => {
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_size) => {
                        graphics.resize_surface(UVec2::new(new_size.width, new_size.height));
                        camera.size = Vec2::new(new_size.width as _, new_size.height as _);
                    }
                    WindowEvent::RedrawRequested => {
                        let surface_texture = match graphics.surface.get_current_texture() {
                            Ok(surface_texture) => surface_texture,
                            Err(wgpu::SurfaceError::Lost) => {
                                graphics.configure_surface();
                                return;
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                *control_flow = ControlFlow::ExitWithCode(1);
                                return;
                            }
                            Err(_) => return,
                        };

                        let surface_view = surface_texture.texture.create_view(&Default::default());

                        shape_renderer.begin();
                        shape_renderer.add(&shape, Default::default());
                        shape_renderer.end();

                        let mut encoder = graphics.context.device().create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("command_encoder"),
                            },
                        );

                        {
                            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("render_pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &surface_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: true,
                                    },
                                })],
                                depth_stencil_attachment: None,
                            });

                            camera_manager.clear();

                            let camera_bind_group =
                                camera_manager.alloc_bind_group(&camera.ortho_matrix());

                            pass.set_bind_group(0, camera_bind_group, &[]);
                            shape_renderer.draw(&mut pass);
                        }

                        graphics.context.queue().submit(Some(encoder.finish()));
                        surface_texture.present();
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    })?;

    Ok(())
}
