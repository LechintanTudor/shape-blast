mod button_bindings;
mod controller;
mod cursor_position;

pub use self::button_bindings::*;
pub use self::controller::*;
pub use self::cursor_position::*;

use crate::gameplay::{BoundingBox, Player, Speed};
use anchor::glam::Vec2;
use sparsey::prelude::*;
use std::ops::Index;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ControllerId(usize);

#[derive(Default, Debug)]
pub struct ControllerManager {
    controllers: Vec<(ButtonBindings, Controller)>,
}

impl ControllerManager {
    pub fn create_controller(&mut self, bindings: ButtonBindings) -> ControllerId {
        let index = self.controllers.len();
        self.controllers.push((bindings, Default::default()));
        ControllerId(index)
    }

    pub fn clear_controllers(&mut self) {
        self.controllers.clear();
    }

    pub fn handle_button_press<B>(&mut self, button: B)
    where
        B: Into<Button>,
    {
        let button = button.into();

        for (bindings, state) in self.controllers.iter_mut() {
            if button == bindings.up {
                state.y_axis.handle_press(ControllerAxis::Negative);
            } else if button == bindings.down {
                state.y_axis.handle_press(ControllerAxis::Positive);
            } else if button == bindings.left {
                state.x_axis.handle_press(ControllerAxis::Negative);
            } else if button == bindings.right {
                state.x_axis.handle_press(ControllerAxis::Positive);
            } else if button == bindings.primary_action {
                state.primary_action = true;
            }
        }
    }

    pub fn handle_button_release<B>(&mut self, button: B)
    where
        B: Into<Button>,
    {
        let button = button.into();

        for (bindings, state) in self.controllers.iter_mut() {
            if button == bindings.up {
                state.y_axis.handle_release(ControllerAxis::Negative);
            } else if button == bindings.down {
                state.y_axis.handle_release(ControllerAxis::Positive);
            } else if button == bindings.left {
                state.x_axis.handle_release(ControllerAxis::Negative);
            } else if button == bindings.right {
                state.x_axis.handle_release(ControllerAxis::Positive);
            } else if button == bindings.primary_action {
                state.primary_action = false;
            }
        }
    }
}

impl Index<ControllerId> for ControllerManager {
    type Output = Controller;

    fn index(&self, id: ControllerId) -> &Self::Output {
        &self.controllers[id.0].1
    }
}

pub fn controller_system(
    controller_manager: Res<ControllerManager>,
    controller_ids: Comp<ControllerId>,
    mut speeds: CompMut<Speed>,
) {
    (&controller_ids, &mut speeds).for_each(|(controller_id, speed)| {
        let controller = &controller_manager[*controller_id];
        speed.direction = controller.direction();
    });
}

pub fn cursor_system(
    mut players: CompMut<Player>,
    bounds: Comp<BoundingBox>,
    cursor_position: Res<CursorPosition>,
) {
    (&mut players, &bounds).for_each(|(player, bounds)| {
        player.angle = Vec2::NEG_Y.angle_between(cursor_position.get() - bounds.position);
    });
}
