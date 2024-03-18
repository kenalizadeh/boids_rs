use crate::{CollisionVolume, Configuration};
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                debug_volumes_system.run_if(volume_debug),
                config_update_system,
            ),
        );
    }
}

fn volume_debug(config: Res<Configuration>) -> bool {
    config.volume_debug
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

fn config_update_system(mut config: ResMut<Configuration>, key_input: Res<ButtonInput<KeyCode>>) {
    if key_input.just_pressed(KeyCode::Space) {
        config.flock_debug = !config.flock_debug;
        config.volume_debug = !config.volume_debug;
        config.ray_debug = !config.ray_debug;
    }
}
