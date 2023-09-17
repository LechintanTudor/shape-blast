use crate::gameplay::{movement_system, Bounds, Player, Speed};
use crate::graphics::Shapes;
use crate::input::{controller_system, ControllerManager, Keybindings};
use anchor::game::{Context, Game, GameResult};
use anchor::glam::Vec2;
use anchor::graphics::{AsDrawable, Canvas, Drawable};
use anchor::winit::event::KeyEvent;
use anchor::winit::keyboard::KeyCode;
use sparsey::prelude::*;

pub struct ShapeBlast {
    shapes: Shapes,
    world: World,
    resources: Resources,
    fixed_update_schedule: Schedule,
}

impl ShapeBlast {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let fixed_update_schedule = Schedule::builder()
            .add_system(controller_system)
            .add_system(movement_system)
            .build();

        Ok(Self {
            shapes: Shapes::new(&ctx.graphics.wgpu)?,
            world: Default::default(),
            resources: Default::default(),
            fixed_update_schedule,
        })
    }
}

impl Game for ShapeBlast {
    fn on_init(&mut self, _ctx: &mut Context) -> GameResult {
        self.resources.insert(ControllerManager::default());
        let mut controller_manager = self.resources.borrow_mut::<ControllerManager>();

        self.world.register::<Player>();
        self.fixed_update_schedule.set_up(&mut self.world);

        self.world.create((
            Player,
            Bounds::from_position_and_size(Vec2::splat(300.0), Vec2::splat(100.0)),
            Speed::from_velocity(5.0),
            controller_manager.create_controller(Keybindings {
                up: KeyCode::KeyW,
                down: KeyCode::KeyS,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
                primary_action: KeyCode::Space,
            }),
        ));

        Ok(())
    }

    fn on_key_event(&mut self, _ctx: &mut Context, event: KeyEvent, _is_synthetic: bool) {
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

    fn fixed_update(&mut self, _ctx: &mut Context) -> GameResult {
        self.fixed_update_schedule
            .run(&mut self.world, &mut self.resources);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::new(ctx);

        self.world
            .run(|players: Comp<Player>, bounds: Comp<Bounds>| {
                let mut player_bounds = (&bounds)
                    .include(&players)
                    .iter()
                    .with_entity()
                    .collect::<Vec<_>>();

                player_bounds.sort_by_key(|(entity, _)| *entity);

                for (_, bounds) in player_bounds.iter() {
                    self.shapes
                        .player
                        .as_drawable()
                        .translation(bounds.position)
                        .draw(&mut canvas);
                }
            });

        canvas.present();
        Ok(())
    }
}
