use crate::BoidMovement;
use bevy::{
    ecs::system::{Query, Res},
    prelude::*,
    time::Time,
    transform::components::Transform,
};
use bevy_math::Quat;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, boids_movement_system);
    }
}

fn boids_movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut BoidMovement)>) {
    for (mut transform, mut movement) in &mut query {
        if let Some(velocity) = movement.target_velocity {
            let angle = velocity.to_angle() * movement.rotation_speed * time.delta_seconds();
            let boid_fowrad_vec = (transform.rotation * Vec3::Y).xy();
            if (boid_fowrad_vec.to_angle() - angle).abs() < f32::EPSILON {
                info!("movement early out");
                movement.target_velocity = Option::None;
                continue;
            }
            transform.rotate(Quat::from_rotation_z(angle));
            let translation_delta =
                transform.rotation * Vec3::Y * velocity.length() * time.delta_seconds();
            transform.translation += translation_delta;
            movement.velocity = (transform.rotation * Vec3::Y).xy() * velocity.length();
        }
    }
}
