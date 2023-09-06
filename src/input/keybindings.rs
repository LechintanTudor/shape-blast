use serde::Deserialize;
use winit::keyboard::KeyCode;

#[derive(Clone, Debug, Deserialize)]
pub struct Keybindings {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub primary_action: KeyCode,
}
