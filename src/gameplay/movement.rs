use anchor::glam::Vec2;
use sparsey::query::Query;
use sparsey::world::{Comp, CompMut};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub position: Vec2,
    pub half_size: Vec2,
}

impl BoundingBox {
    pub fn from_position_and_size(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            half_size: size * 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Speed {
    pub direction: Vec2,
    pub velocity: f32,
}

impl Speed {
    pub fn from_velocity(velocity: f32) -> Self {
        Self {
            direction: Vec2::ZERO,
            velocity,
        }
    }

    pub fn displacement(&self) -> Vec2 {
        self.direction * self.velocity
    }
}

pub fn movement_system(mut bounds: CompMut<BoundingBox>, speeds: Comp<Speed>) {
    (&mut bounds, &speeds).for_each(|(bounds, speed)| {
        bounds.position += speed.displacement();
    });
}
