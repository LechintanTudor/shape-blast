use anchor::winit::event::MouseButton;
use anchor::winit::keyboard::KeyCode;
use derive_more::From;
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, From, Debug, Deserialize)]
pub enum Button {
    Key(KeyCode),
    Mouse(MouseButton),
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct ButtonBindings {
    pub up: Button,
    pub down: Button,
    pub left: Button,
    pub right: Button,
    pub primary_action: Button,
}
