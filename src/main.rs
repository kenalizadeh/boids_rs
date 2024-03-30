use bevy::{
    prelude::*,
    window::{close_on_esc, WindowResolution},
};
mod plugins;
use plugins::{
    movement::MovementPlugin,
    rules::RulesPlugin,
    startup::{StartupPlugin, INITIAL_WINDOW_SIZE},
};

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(INITIAL_WINDOW_SIZE.x, INITIAL_WINDOW_SIZE.y),
                title: "Boids Demo".into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(StartupPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(RulesPlugin)
        .add_systems(Update, close_on_esc)
        .run();
}
