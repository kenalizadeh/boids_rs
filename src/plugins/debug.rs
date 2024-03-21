use crate::{BoidFlock, BoidMovement, CollisionVolume, Configuration};
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                debug_flock_system.run_if(flock_debug),
                debug_volumes_system.run_if(volume_debug),
                debug_movement_system.run_if(movement_debug),
                config_update_system,
            ),
        );
    }
}

fn flock_debug(config: Res<Configuration>) -> bool {
    config.flock_debug
}

fn volume_debug(config: Res<Configuration>) -> bool {
    config.volume_debug
}

fn movement_debug(config: Res<Configuration>) -> bool {
    config.movement_debug
}

fn debug_flock_system(mut gizmos: Gizmos, query: Query<(&BoidFlock, &Transform)>) {
    for (flock, transform) in &query {
        gizmos.circle_2d(transform.translation.xy(), flock.radius, Color::FUCHSIA);
        gizmos.line_2d(
            transform.translation.xy(),
            transform.translation.xy() + flock.direction * 2.,
            Color::MAROON,
        )
    }
}

fn debug_volumes_system(mut gizmos: Gizmos, query: Query<(&CollisionVolume, &Transform)>) {
    for (volume, transform) in query.iter() {
        gizmos.rect_2d(
            transform.translation.xy(),
            transform.rotation.to_euler(EulerRot::YXZ).2,
            volume.shape.size(),
            Color::PINK,
        )
    }
}

fn debug_movement_system(mut gizmos: Gizmos, query: Query<(&Transform, &BoidMovement)>) {
    for (transform, movement) in &query {
        let velocity = movement.velocity;
        let start = transform.translation.xy();
        gizmos.line_2d(start, start + velocity * 2., Color::ORANGE_RED);
    }
}

fn config_update_system(mut config: ResMut<Configuration>, key_input: Res<ButtonInput<KeyCode>>) {
    if key_input.just_pressed(KeyCode::Space) {
        config.flock_debug = !config.flock_debug;
        // config.movement_debug = !config.movement_debug;
        // config.volume_debug = !config.volume_debug;
        // config.ray_debug = !config.ray_debug;
    }
}
