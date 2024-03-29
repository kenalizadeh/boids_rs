use bevy::ecs::{component::Component, system::Resource};
use bevy_math::{primitives::Rectangle, Vec2};

#[derive(Debug)]
pub struct RectFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectFrame {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            width: w,
            height: h,
        }
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn size(&self) -> Rectangle {
        Rectangle::new(self.width, self.height)
    }
}

#[derive(Component, Default)]
pub struct BoidMovement {
    pub speed: f32,
    pub target_angle: f32,
    pub rotation_speed: f32,
}

impl BoidMovement {
    pub fn new(speed: f32, target_angle: f32, rotation_speed: f32) -> Self {
        Self {
            speed,
            target_angle,
            rotation_speed,
        }
    }
}

#[derive(Component)]
pub enum Wall {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Resource, Default)]
pub struct Configuration {
    pub movement_debug: bool,
    pub flock_debug: bool,
    pub volume_debug: bool,
    pub ray_debug: bool,
}

#[derive(Component)]
pub struct SeparationRule {
    pub id: usize,
    pub radius: f32,
    // between 0.0 and 1.0
    // 0 means off
    pub factor: f32,
    pub velocity: Vec2,
}

impl SeparationRule {
    pub fn new(id: usize, radius: f32, factor: f32, velocity: Vec2) -> Self {
        Self {
            id,
            radius,
            factor,
            velocity,
        }
    }
}

#[derive(Component)]
pub struct AlignmentRule {
    pub id: usize,
    pub radius: f32,
    // between 0.0 and 1.0
    // 0 means off
    pub factor: f32,
    pub velocity: Vec2,
}

impl AlignmentRule {
    pub fn new(id: usize, radius: f32, factor: f32, velocity: Vec2) -> Self {
        Self {
            id,
            radius,
            factor,
            velocity,
        }
    }
}

#[derive(Component)]
pub struct CohesionRule {
    pub id: usize,
    pub radius: f32,
    // between 0.0 and 1.0
    // 0 means off
    pub factor: f32,
    pub velocity: Vec2,
}

impl CohesionRule {
    pub fn new(id: usize, radius: f32, factor: f32, velocity: Vec2) -> Self {
        Self {
            id,
            radius,
            factor,
            velocity,
        }
    }
}
