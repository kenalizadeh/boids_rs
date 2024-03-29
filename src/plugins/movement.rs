use crate::BoidMovement;
use bevy::{
    ecs::system::{Query, Res},
    prelude::*,
    time::Time,
    transform::components::Transform,
};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (boids_rotation_system, boids_movement_system).chain(),
        );
    }
}

fn boids_rotation_system(
    mut gizmos: Gizmos,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &BoidMovement)>,
) {
    for (mut transform, movement) in &mut query {
        let curr_vel = (transform.rotation * Vec3::Y).xy();
        let target_vel = Vec2::from_angle(movement.target_angle).normalize();
        let target_vel_dot = curr_vel.normalize().dot(target_vel);

        if (target_vel_dot - 1.).abs() < f32::EPSILON {
            continue;
        }

        let right_vector = (transform.rotation * Vec3::X).xy();

        let right_vector_dot = right_vector.dot(target_vel);

        let rotation_sign = -f32::copysign(1.0, right_vector_dot);

        let max_angle = target_vel_dot.clamp(-1.0, 1.0).acos();

        let rotation_angle =
            rotation_sign * (movement.rotation_speed * time.delta_seconds()).min(max_angle);

        transform.rotate_z(rotation_angle);

        // gizmos.arrow_2d(
        //     transform.translation.xy(),
        //     transform.translation.xy() + Vec2::from_angle(movement.target_angle).normalize() * 30.,
        //     Color::YELLOW,
        // );
    }
}

fn boids_movement_system(
    mut gizmos: Gizmos,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<BoidMovement>>,
) {
    for mut transform in &mut query {
        // let translation_delta = transform.rotation * Vec3::Y * 30. * time.delta_seconds();
        // transform.translation += translation_delta;

        // get the ship's forward vector by applying the current rotation to the ships initial facing
        // vector
        let movement_direction = transform.rotation * Vec3::Y;
        // get the distance the ship will move based on direction, the ship's movement speed and delta
        // time
        let movement_distance = 30. * time.delta_seconds();
        // create the change in translation using the new movement direction and distance
        let translation_delta = movement_direction * movement_distance;
        // update the ship translation with our new translation delta
        transform.translation += translation_delta;

        gizmos.arrow_2d(
            transform.translation.xy(),
            transform.translation.xy() + movement_direction.xy().normalize() * 30.,
            Color::CRIMSON,
        );
    }
}
