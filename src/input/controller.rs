use glam::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ControllerAxis {
    #[default]
    Neutral,
    Negative,
    Positive,
}

impl ControllerAxis {
    pub fn handle_press(&mut self, value: ControllerAxis) {
        *self = value;
    }

    pub fn handle_release(&mut self, value: ControllerAxis) {
        if self == &value {
            *self = Self::Neutral;
        }
    }

    pub fn value(&self) -> f32 {
        match self {
            Self::Neutral => 0.0,
            Self::Negative => -1.0,
            Self::Positive => 1.0,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Controller {
    pub x_axis: ControllerAxis,
    pub y_axis: ControllerAxis,
    pub primary_action: bool,
}

impl Controller {
    pub fn direction(&self) -> Vec2 {
        Vec2::new(self.x_axis.value(), self.y_axis.value()).normalize_or_zero()
    }
}
