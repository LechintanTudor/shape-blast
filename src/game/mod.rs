mod utils;

use crate::game::utils::load_from_ron_file;
use crate::gameplay::{movement_system, BoundingBox, Player, Speed};
use crate::graphics::Shapes;
use crate::input::{
    controller_system, cursor_system, ButtonBindings, ControllerManager, CursorPosition,
};
use anchor::game::{Context, Game, GameResult};
use anchor::glam::{DVec2, Vec2};
use anchor::graphics::{AsDrawable, Canvas, Drawable};
use anchor::winit::event::KeyEvent;
use sparsey::prelude::*;

pub struct ShapeBlast {
    bindings: ButtonBindings,
    shapes: Shapes,
    world: World,
    resources: Resources,
    fixed_update_schedule: Schedule,
}

impl ShapeBlast {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let bindings = load_from_ron_file("assets/bindings.ron")?;

        let fixed_update_schedule = Schedule::builder()
            .add_system(controller_system)
            .add_system(cursor_system)
            .add_system(movement_system)
            .build();

        Ok(Self {
            bindings,
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
        self.resources.insert(CursorPosition::default());

        let mut controller_manager = self.resources.borrow_mut::<ControllerManager>();

        self.world.register::<Player>();
        self.fixed_update_schedule.set_up(&mut self.world);

        self.world.create((
            Player::default(),
            BoundingBox::from_position_and_size(Vec2::splat(300.0), Vec2::splat(100.0)),
            Speed::from_velocity(5.0),
            controller_manager.create_controller(self.bindings.clone()),
        ));

        Ok(())
    }

    fn on_key_event(&mut self, _ctx: &mut Context, event: KeyEvent, _is_synthetic: bool) {
        if event.repeat {
            return;
        }

        let mut controller_manager = self.resources.borrow_mut::<ControllerManager>();

        if event.state.is_pressed() {
            controller_manager.handle_button_press(event.physical_key);
        } else {
            controller_manager.handle_button_release(event.physical_key);
        }
    }

    fn on_cursor_move(&mut self, _ctx: &mut Context, new_cursor_position: DVec2) {
        let mut cursor_position = self.resources.borrow_mut::<CursorPosition>();
        cursor_position.set(new_cursor_position.as_vec2());
    }

    fn fixed_update(&mut self, _ctx: &mut Context) -> GameResult {
        self.fixed_update_schedule
            .run(&mut self.world, &mut self.resources);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::new(ctx);

        self.world
            .run(|players: Comp<Player>, bounds: Comp<BoundingBox>| {
                let mut player_bounds =
                    (&players, &bounds).iter().with_entity().collect::<Vec<_>>();

                player_bounds.sort_by_key(|(entity, _)| *entity);

                for (_, (player, bounds)) in player_bounds.iter() {
                    self.shapes
                        .player
                        .as_drawable()
                        .translation(bounds.position)
                        .rotation(player.angle)
                        .draw(&mut canvas);
                }
            });

        canvas.present();
        Ok(())
    }
}
