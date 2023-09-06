mod controller;
mod keybindings;

pub use self::controller::*;
pub use self::keybindings::*;
use std::ops::Index;
use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ControllerId(usize);

#[derive(Default, Debug)]
pub struct ControllerManager {
    controllers: Vec<(Keybindings, Controller)>,
}

impl ControllerManager {
    pub fn create_controller(&mut self, keybindings: Keybindings) -> ControllerId {
        let index = self.controllers.len();
        self.controllers.push((keybindings, Default::default()));
        ControllerId(index)
    }

    pub fn clear_controllers(&mut self) {
        self.controllers.clear();
    }

    pub fn handle_key_press(&mut self, key: KeyCode) {
        for (keybindings, state) in self.controllers.iter_mut() {
            if key == keybindings.up {
                state.y_axis.handle_press(ControllerAxis::Negative);
            } else if key == keybindings.down {
                state.y_axis.handle_press(ControllerAxis::Positive);
            } else if key == keybindings.left {
                state.x_axis.handle_press(ControllerAxis::Negative);
            } else if key == keybindings.right {
                state.x_axis.handle_press(ControllerAxis::Positive);
            } else if key == keybindings.primary_action {
                state.primary_action = true;
            }
        }
    }

    pub fn handle_key_release(&mut self, key: KeyCode) {
        for (keybindings, state) in self.controllers.iter_mut() {
            if key == keybindings.up {
                state.y_axis.handle_release(ControllerAxis::Negative);
            } else if key == keybindings.down {
                state.y_axis.handle_release(ControllerAxis::Positive);
            } else if key == keybindings.left {
                state.x_axis.handle_release(ControllerAxis::Negative);
            } else if key == keybindings.right {
                state.x_axis.handle_release(ControllerAxis::Positive);
            } else if key == keybindings.primary_action {
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
