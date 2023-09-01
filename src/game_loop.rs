use crate::graphics::Graphics;
use glam::UVec2;
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    let window = WindowBuilder::new()
        .with_title("Shape Blast")
        .with_inner_size(LogicalSize::new(640.0, 480.0))
        .with_min_inner_size(LogicalSize::new(320.0, 240.0))
        .build(&event_loop)?;

    let mut graphics = unsafe { Graphics::new(&window)? };

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

                        let mut encoder = graphics.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("command_encoder"),
                            },
                        );

                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("render_pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &surface_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });

                        graphics.queue.submit(Some(encoder.finish()));
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
