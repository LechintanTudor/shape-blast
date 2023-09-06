mod frame_timer;

use self::frame_timer::*;
use crate::gameplay::{movement_system, Bounds, Player, Speed};
use crate::graphics::shape::ShapeParams;
use crate::graphics::{Camera, GraphicsManager, Shapes};
use crate::input::{controller_system, ControllerManager, Keybindings};
use glam::{Affine2, UVec2, Vec2};
use sparsey::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowBuilder};

const WNIDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 500.0;

pub struct Game {
    graphics: GraphicsManager,
    window: Window,
    frame_timer: FrameTimer,
    world: World,
    resources: Resources,
    fixed_update_schedule: Schedule,
    camera: Camera,
    shapes: Shapes,
}

impl Game {
    pub fn run() -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;

        let window = WindowBuilder::new()
            .with_title("Shape Blast")
            .with_inner_size(LogicalSize::new(WNIDOW_WIDTH, WINDOW_HEIGHT))
            .with_min_inner_size(LogicalSize::new(WNIDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0))
            .with_visible(false)
            .build(&event_loop)?;

        let graphics = unsafe { GraphicsManager::new(&window)? };

        let fixed_update_schedule = Schedule::builder()
            .add_system(controller_system)
            .add_system(movement_system)
            .build();

        let mut world = World::default();
        world.register::<Player>();
        world.register::<Bounds>();
        fixed_update_schedule.set_up(&mut world);

        let mut resources = Resources::default();
        resources.insert(ControllerManager::default());

        let window_size = {
            let window_size = window.inner_size();
            UVec2::new(window_size.width, window_size.height).as_vec2()
        };
        let camera = Camera::from_size(window_size);

        let shapes = Shapes::new(graphics.context.device())?;

        let mut game = Game {
            graphics,
            window,
            frame_timer: Default::default(),
            world,
            resources,
            fixed_update_schedule,
            camera,
            shapes,
        };

        event_loop.run(|event, _, control_flow| {
            match event {
                Event::NewEvents(StartCause::Init) => {
                    game.init();
                    game.window.set_visible(true);
                }
                Event::NewEvents(StartCause::Poll) => {
                    game.frame_timer.start_frame();

                    if game.frame_timer.fixed_update() {
                        game.fixed_update();
                    }

                    game.window.request_redraw();
                }
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            game.handle_key_event(event);
                        }
                        WindowEvent::Resized(new_size) => {
                            game.handle_resize(UVec2::new(new_size.width, new_size.height));
                        }
                        WindowEvent::RedrawRequested => {
                            game.draw();
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        })?;

        Ok(())
    }

    fn init(&mut self) {
        let mut controller_manager = self.resources.borrow_mut::<ControllerManager>();

        self.world.create((
            Player,
            Bounds::from_position_and_size(Vec2::ZERO, Vec2::splat(100.0)),
            Speed::from_velocity(5.0),
            controller_manager.create_controller(Keybindings {
                up: KeyCode::KeyW,
                down: KeyCode::KeyS,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
                primary_action: KeyCode::Space,
            }),
        ));
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        if event.repeat {
            return;
        }

        let mut controller_manager = self.resources.borrow_mut::<ControllerManager>();

        if event.state.is_pressed() {
            controller_manager.handle_key_press(event.physical_key);
        } else {
            controller_manager.handle_key_release(event.physical_key);
        }
    }

    fn handle_resize(&mut self, new_size: UVec2) {
        self.graphics.resize_surface(new_size);
        self.camera.size = new_size.as_vec2();
    }

    fn fixed_update(&mut self) {
        self.fixed_update_schedule
            .run(&mut self.world, &mut self.resources);
    }

    fn draw(&mut self) {
        let surface_texture = match self.graphics.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(wgpu::SurfaceError::Lost) => {
                self.graphics.configure_surface();
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                return;
            }
            Err(_) => return,
        };

        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self.graphics.context.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            },
        );

        self.world
            .run(|players: Comp<Player>, bounds: Comp<Bounds>| {
                self.graphics.shape_renderer.begin();

                let mut bounds = (&bounds)
                    .include(&players)
                    .iter()
                    .with_entity()
                    .collect::<Vec<_>>();

                bounds.sort_by_key(|(entity, _)| *entity);

                for (_, bounds) in bounds.iter() {
                    self.graphics.shape_renderer.add(
                        &self.shapes.player,
                        ShapeParams {
                            transform: Affine2::from_translation(bounds.position),
                            ..Default::default()
                        },
                    );
                }

                self.graphics.shape_renderer.end();
            });

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

            self.graphics.camera_manager.clear();

            let camera_bind_group = self
                .graphics
                .camera_manager
                .alloc_bind_group(&self.camera.ortho_matrix());

            pass.set_bind_group(0, camera_bind_group, &[]);
            self.graphics.shape_renderer.draw(&mut pass);
        }

        self.graphics.context.queue().submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
