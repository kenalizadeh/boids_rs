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
            (
                boids_rotation_system,
                boids_forward_movement_system,
                boids_teleport_system,
            )
                .chain(),
        );
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

fn boids_rotation_system(time: Res<Time>, mut query: Query<(&mut Transform, &BoidMovement)>) {
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
    }
}

fn boids_forward_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &BoidMovement), With<BoidMovement>>,
) {
    for (mut transform, movement) in &mut query {
        let movement_direction = transform.rotation * Vec3::Y;
        let movement_distance = movement.speed * time.delta_seconds();
        let translation_delta = movement_direction * movement_distance;
        transform.translation += translation_delta;
    }
}

fn boids_teleport_system(
    mut query: Query<&mut Transform, With<BoidMovement>>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();
    let left_bound: f32 = -(window.width() / 2.);
    let right_bound: f32 = window.width() / 2.;
    let bottom_bound: f32 = -(window.height() / 2.);
    let top_bound: f32 = window.height() / 2.;

    for mut transform in &mut query {
        let center = transform.translation.xy();

        match center.x {
            x if x > right_bound => {
                transform.translation.x = left_bound;
            }
            x if x < left_bound => {
                transform.translation.x = right_bound;
            }
            _ => (),
        }

        match center.y {
            y if y > top_bound => {
                transform.translation.y = bottom_bound;
            }
            y if y < bottom_bound => {
                transform.translation.y = top_bound;
            }
            _ => (),
        }
    }
}
