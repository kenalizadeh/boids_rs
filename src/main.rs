use bevy::{
    prelude::*,
    utils::HashMap,
    window::{close_on_esc, WindowResized, WindowResolution},
};
mod components;
mod plugins;
use components::*;
use plugins::{
    debug::DebugPlugin,
    movement::MovementPlugin,
    setup::{StartupPlugin, INITIAL_WINDOW_SIZE},
};

/// Raycast
const PI: f32 = std::f32::consts::PI;
const RAYCAST_FOV: f32 = 270. * (PI / 180_f32);
const RAY_COUNT: u8 = 15;
const RAYCAST_DIST: f32 = 100.;

#[derive(Resource)]
struct BoidDebug {
    id: usize,
}

impl BoidDebug {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .insert_resource(Configuration::default())
        .insert_resource(BoidDebug::new(0))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(INITIAL_WINDOW_SIZE.x, INITIAL_WINDOW_SIZE.y),
                title: "Boids Demo".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(StartupPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(DebugPlugin)
        .add_systems(
            FixedUpdate,
            (
                close_on_esc,
                separation_system,
                alignment_system,
                cohesion_system,
                velocity_system,
                boids_teleport_system,
            )
                .chain(),
        )
        .run();
}

fn separation_system(
    mut gizmos: Gizmos,
    mut query: Query<(&GlobalTransform, &mut SeparationRule)>,
    debug: Res<BoidDebug>,
) {
    let mut velocity_map: HashMap<usize, Vec2> = HashMap::new();
    for (current_transform, current_separation) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut velocity = Vec2::ZERO;
        // max magnitude
        const MAGNITUDE: f32 = 60.;

        if debug.id == current_separation.id {
            gizmos.circle_2d(current_center, current_separation.radius, Color::CYAN);
        }

        for (transform, separation) in &query {
            if separation.id == current_separation.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_separation.radius {
                continue;
            }

            if debug.id == current_separation.id {
                // gizmos.arrow_2d(center, center + movement.velocity * 30., Color::LIME_GREEN);
                gizmos.line_2d(current_center, center, Color::DARK_GREEN);
            }

            // adding vectors gives us the attraction velocity, subtracting does the opposite.
            let separation_velocity = current_center - center;
            let weight = (current_separation.radius - distance) / current_separation.radius;
            let weighted_velocity = separation_velocity.normalize() * MAGNITUDE * weight;

            velocity += weighted_velocity;
            nearby_boid_count += 1.;
        }

        if nearby_boid_count > 0. {
            velocity /= nearby_boid_count;
            velocity *= current_separation.factor;

            if debug.id == current_separation.id {
                gizmos.arrow_2d(current_center, current_center + velocity, Color::BLUE);
            }

            velocity_map.insert(current_separation.id, velocity);
        }
    }

    for (_, mut separation) in &mut query {
        if let Some(vel) = velocity_map.get(&separation.id) {
            separation.velocity = *vel;
        }
    }
}

fn alignment_system(
    mut gizmos: Gizmos,
    mut query: Query<(&Transform, &mut AlignmentRule)>,
    debug: Res<BoidDebug>,
) {
    let mut velocity_map: HashMap<usize, Vec2> = HashMap::new();
    for (current_transform, current_alignment) in &query {
        let current_center = current_transform.translation.xy();
        let mut nearby_boid_count = 0_f32;
        let mut velocity = Vec2::ZERO;

        if debug.id == current_alignment.id {
            gizmos.circle_2d(current_center, current_alignment.radius, Color::CYAN);
        }

        for (transform, alignment) in &query {
            if alignment.id == current_alignment.id {
                continue;
            }

            let center = transform.translation.xy();
            let distance = current_center.distance(center);
            if distance > current_alignment.radius {
                continue;
            }

            let boid_velocity = (transform.rotation * Vec3::Y).xy();
            if debug.id == current_alignment.id {
                gizmos.arrow_2d(
                    center,
                    center + boid_velocity.normalize() * 30.,
                    Color::LIME_GREEN,
                );
                gizmos.line_2d(current_center, center, Color::DARK_GREEN);
            }

            velocity += boid_velocity;
            nearby_boid_count += 1.;
        }

        if nearby_boid_count > 0. {
            velocity /= nearby_boid_count;
            velocity *= current_alignment.factor;

            if debug.id == current_alignment.id {
                gizmos.arrow_2d(
                    current_center,
                    current_center + velocity.normalize() * 50.,
                    Color::BLUE,
                );
            }

            velocity_map.insert(current_alignment.id, velocity);
        }
    }

    for (_, mut alignment) in &mut query {
        if let Some(vel) = velocity_map.get(&alignment.id) {
            alignment.velocity = *vel;
        }
    }
}

fn cohesion_system(
    mut gizmos: Gizmos,
    mut query: Query<(&GlobalTransform, &mut CohesionRule)>,
    debug: Res<BoidDebug>,
) {
    let mut velocity_map: HashMap<usize, Vec2> = HashMap::new();
    for (current_transform, current_cohesion) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut center_of_mass = current_center;
        let mut boid_positions: Vec<Vec2> = vec![];

        if debug.id == current_cohesion.id {
            gizmos.circle_2d(current_center, current_cohesion.radius, Color::CYAN);
        }

        for (transform, cohesion) in &query {
            if cohesion.id == current_cohesion.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_cohesion.radius {
                continue;
            }

            center_of_mass += center;
            nearby_boid_count += 1.;

            boid_positions.push(center);
        }

        if nearby_boid_count > 0. {
            center_of_mass -= current_center;
            center_of_mass /= nearby_boid_count;

            if debug.id == current_cohesion.id {
                gizmos.circle_2d(center_of_mass, 1., Color::CRIMSON);
                for boid_pos in boid_positions {
                    gizmos.line_2d(center_of_mass, boid_pos, Color::DARK_GREEN);
                }
            }

            let com_vector = center_of_mass - current_center;
            let com_velocity = com_vector.normalize() * current_cohesion.factor;

            if debug.id == current_cohesion.id {
                gizmos.arrow_2d(current_center, center_of_mass, Color::BLUE);
            }

            velocity_map.insert(current_cohesion.id, com_velocity);
        }
    }

    for (_, mut alignment) in &mut query {
        if let Some(vel) = velocity_map.get(&alignment.id) {
            alignment.velocity = *vel;
        }
    }
}

fn velocity_system(
    mut gizmos: Gizmos,
    mut query: Query<(
        &Transform,
        &mut BoidMovement,
        &SeparationRule,
        &AlignmentRule,
        &CohesionRule,
    )>,
    debug: Res<BoidDebug>,
) {
    for (transform, mut movement, separation, alignment, cohesion) in &mut query {
        let velocities = [separation.velocity, alignment.velocity, cohesion.velocity];
        let velocity: Vec2 = velocities.iter().map(|v| v.normalize()).sum();
        let center = transform.translation.xy();

        if debug.id == separation.id {
            gizmos.arrow_2d(center, center + velocity.normalize() * 30., Color::PURPLE);
        }

        if !velocity.is_nan() {
            movement.target_angle = velocity.to_angle();
        }
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

// fn debug_boid_spawn_system(
//     window_query: Query<&Window>,
//     camera_query: Query<(&Camera, &GlobalTransform)>,
//     key_input: Res<ButtonInput<KeyCode>>,
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
// ) {
//     if key_input.just_pressed(KeyCode::KeyG) {
//         let (camera, camera_transform) = camera_query.single();
//
//         let Some(cursor_pos) = window_query.single().cursor_position() else {
//             return;
//         };
//
//         let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
//             return;
//         };
//
//         let boid_handle = asset_server.load("textures/berd.png");
//         let direction_degrees = (fastrand::f32() * 360.0).to_radians();
//         let target_degrees = (fastrand::f32() * 360.0).to_radians();
//         commands.spawn((
//             SpriteBundle {
//                 texture: boid_handle.clone(),
//                 transform: Transform::from_xyz(point.x, point.y, 0.0)
//                     .with_rotation(Quat::from_rotation_z(direction_degrees)),
//                 ..default()
//             },
//             SeparationRule::new(120, 100., 1., Vec2::ZERO),
//             AlignmentRule::new(120, 100., 1., Vec2::ZERO),
//             CohesionRule::new(120, 100., 1., Vec2::ZERO),
//             BoidMovement::new(target_degrees, std::f32::consts::PI),
//         ));
//     }
// }
