use bevy::{
    prelude::*,
    utils::HashMap,
    window::{close_on_esc, WindowResolution},
};
use bevy_math::bounding::{Bounded2d, RayCast2d};
mod components;
mod plugins;
use components::*;
use plugins::{
    debug::DebugPlugin,
    movement::MovementPlugin,
    setup::{StartupPlugin, WINDOW_SIZE},
};

/// Raycast
const PI: f32 = std::f32::consts::PI;
const RAYCAST_FOV: f32 = 270. * (PI / 180_f32);
const RAY_COUNT: u8 = 31;
const RAYCAST_DIST: f32 = 200.;

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .insert_resource(Configuration::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y),
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
            (close_on_esc, boids_raycast_drawing_system).chain(),
        )
        .run();
}

fn cast_ray_and_check(
    idx: u8,
    gizmos: &mut Gizmos,
    center: Vec2,
    direction_angle: f32,
    coll_volume: &CollisionVolume,
    volumes_query: &Query<(
        &mut Transform,
        &CollisionVolume,
        Option<&mut BoidMovement>,
        // Option<&mut BoidFlock>,
    )>,
    debug: bool,
) -> (bool, f32) {
    let ray_spacing = RAYCAST_FOV / (RAY_COUNT - 1) as f32;
    let idx_even = idx % 2 == 0;
    let div = (idx as i8 / 2) as f32;
    let mul: f32 = if idx_even { -1. } else { 1. };
    let ray_angle = direction_angle + mul * (idx as f32 * ray_spacing - (div * ray_spacing));
    let ray_vec = Vec2::from_angle(ray_angle);
    let ray_cast = RayCast2d::new(center, Direction2d::new(ray_vec).unwrap(), RAYCAST_DIST);

    let mut hits = false;
    for (transform, volume, _) in volumes_query.iter() {
        if volume.id == coll_volume.id {
            continue;
        }
        let aabb = volume.shape.aabb_2d(
            transform.translation.xy(),
            transform.rotation.to_euler(EulerRot::YXZ).2,
        );
        let coll = ray_cast.aabb_intersection_at(&aabb).is_some();
        if coll {
            hits = true;
            break;
        }
    }
    if debug {
        gizmos.line_2d(
            ray_cast.ray.origin,
            ray_cast.ray.origin + ray_vec * ray_cast.max,
            if hits {
                Color::CRIMSON
            } else {
                Color::TURQUOISE
            },
        );
    }

    (hits, ray_angle - direction_angle)
}

fn filter_local_flock(
    flock_radius: f32,
    center: Vec2,
    query: Query<(
        &mut Transform,
        &CollisionVolume,
        Option<&mut BoidMovement>,
        Option<&mut BoidFlock>,
    )>,
) {
    let mut vectors: Vec<Vec2> = vec![];
    for (transform, _, movement, _) in query.iter() {
        if let Some(movement) = movement {
            let distance = center.distance(transform.translation.xy());
            if distance <= flock_radius {
                vectors.push(movement.velocity)
            }
        }
    }
}

fn boids_raycast_drawing_system(
    config: ResMut<Configuration>,
    mut gizmos: Gizmos,
    mut query: Query<(
        &mut Transform,
        &CollisionVolume,
        Option<&mut BoidMovement>,
        // Option<&mut BoidFlock>,
    )>,
) {
    let mut map: HashMap<usize, f32> = HashMap::new();
    for (transform, current_volume, movement) in &query {
        // ray-casting
        if let Some(movement) = movement {
            let center = transform.translation.xy();
            let direction_angle = movement.velocity.to_angle();

            if config.ray_debug {
                gizmos.arc_2d(
                    center,
                    PI / 2. - direction_angle,
                    RAYCAST_FOV,
                    RAYCAST_DIST,
                    Color::FUCHSIA.with_a(0.2),
                );
            }
            for idx in 0..RAY_COUNT {
                let (hits, angle) = cast_ray_and_check(
                    idx,
                    &mut gizmos,
                    center,
                    direction_angle,
                    current_volume,
                    &query,
                    config.ray_debug,
                );

                if !hits {
                    map.insert(current_volume.id, angle);
                    break;
                }
            }
            if map.get(&current_volume.id).is_none() {
                map.insert(current_volume.id, PI / 2.);
            }
        }
    }

    for (_, current_volume, movement) in &mut query {
        if let Some(mut movement) = movement {
            if let Some(angle) = map.get(&current_volume.id) {
                let velocity = Vec2::from_angle(*angle) * movement.velocity.length();
                movement.target_velocity = Some(velocity);
            }
        }
    }
}
