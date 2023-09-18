use anchor::glam::Vec2;
use derive_more::{Deref, DerefMut};

#[derive(Clone, Copy, Deref, DerefMut, Default, Debug)]
pub struct CursorPosition(Vec2);

impl CursorPosition {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    pub fn set<P>(&mut self, position: P)
    where
        P: Into<Vec2>,
    {
        self.0 = position.into();
    }

    pub fn get(&self) -> Vec2 {
        self.0
    }
}
