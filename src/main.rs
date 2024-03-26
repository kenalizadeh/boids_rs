use bevy::{
    prelude::*,
    reflect::Map,
    utils::HashMap,
    window::{close_on_esc, WindowResolution},
};
mod components;
mod plugins;
use components::*;
use plugins::{
    debug::DebugPlugin,
    movement::MovementPlugin,
    setup::{StartupPlugin, INITIAL_WINDOW_SIZE},
};

use crate::plugins::setup::BOID_SPEED;

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
        .insert_resource(BoidDebug::new(12))
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
            )
                .chain(),
        )
        .run();
}

fn separation_system(
    mut gizmos: Gizmos,
    mut query: Query<(&GlobalTransform, &BoidMovement, &mut SeparationRule)>,
    debug: Res<BoidDebug>,
) {
    let mut velocity_map: HashMap<usize, Vec2> = HashMap::new();
    for (current_transform, _, current_separation) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut velocity = Vec2::ZERO;
        // max magnitude
        const MAGNITUDE: f32 = 60.;

        if debug.id == current_separation.id {
            gizmos.circle_2d(current_center, current_separation.radius, Color::CYAN);
        }

        for (transform, movement, separation) in &query {
            if separation.id == current_separation.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_separation.radius {
                continue;
            }

            if debug.id == current_separation.id {
                gizmos.arrow_2d(center, center + movement.velocity * 30., Color::LIME_GREEN);
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

    for (_, _, mut separation) in &mut query {
        if let Some(vel) = velocity_map.get(&separation.id) {
            separation.velocity = *vel;
        }
    }
}

fn alignment_system(
    mut gizmos: Gizmos,
    mut query: Query<(&GlobalTransform, &BoidMovement, &mut AlignmentRule)>,
    debug: Res<BoidDebug>,
) {
    let mut velocity_map: HashMap<usize, Vec2> = HashMap::new();
    for (current_transform, _, current_alignment) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut velocity = Vec2::ZERO;

        if debug.id == current_alignment.id {
            gizmos.circle_2d(current_center, current_alignment.radius, Color::CYAN);
        }

        for (transform, movement, alignment) in &query {
            if alignment.id == current_alignment.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_alignment.radius {
                continue;
            }

            if debug.id == current_alignment.id {
                gizmos.arrow_2d(
                    center,
                    center + movement.velocity.normalize() * 30.,
                    Color::LIME_GREEN,
                );
                gizmos.line_2d(current_center, center, Color::DARK_GREEN);
            }

            velocity += movement.velocity;
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

    for (_, _, mut alignment) in &mut query {
        if let Some(vel) = velocity_map.get(&alignment.id) {
            alignment.velocity = *vel;
        }
    }
}

fn cohesion_system(
    mut gizmos: Gizmos,
    mut query: Query<(&GlobalTransform, &BoidMovement, &mut CohesionRule)>,
    debug: Res<BoidDebug>,
) {
    let mut velocity_map: HashMap<usize, Vec2> = HashMap::new();
    for (current_transform, _, current_cohesion) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut center_of_mass = current_center;
        let mut boid_positions: Vec<Vec2> = vec![];

        if debug.id == current_cohesion.id {
            gizmos.circle_2d(current_center, current_cohesion.radius, Color::CYAN);
        }

        for (transform, movement, cohesion) in &query {
            if cohesion.id == current_cohesion.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_cohesion.radius {
                continue;
            }

            if debug.id == current_cohesion.id {
                gizmos.arrow_2d(
                    center,
                    center + Vec2::from_angle(movement.target_angle) * BOID_SPEED,
                    Color::LIME_GREEN,
                );
            }

            center_of_mass += center;
            nearby_boid_count += 1.;

            boid_positions.push(center);
        }

        if nearby_boid_count > 0. {
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

    for (_, _, mut alignment) in &mut query {
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

        dbg!(separation.id, velocity);

        if !velocity.is_nan() {
            movement.target_angle = velocity.to_angle();
        }
    }
}

// fn boids_flock_system(
//     mut gizmos: Gizmos,
//     config: Res<Configuration>,
//     mut query: Query<(
//         &mut Transform,
//         &mut BoidFlock,
//         &CollisionVolume,
//         &mut BoidMovement,
//     )>,
// ) {
//     let mut map: HashMap<usize, f32> = HashMap::new();
//     for (transform, flock, coll_volume, _) in &query {
//         let center = transform.translation.xy();
//
//         let mut directions: Vec<f64> = vec![];
//         for (t, _, cv, mvm) in &query {
//             if cv.id == coll_volume.id {
//                 continue;
//             }
//
//             let distance = center.distance(t.translation.xy());
//
//             if distance <= flock.radius {
//                 directions.push(mvm.velocity.to_angle().into());
//                 if config.flock_debug {
//                     gizmos.line_2d(center, t.translation.xy(), Color::RED);
//                 }
//             }
//         }
//
//         if directions.is_empty() {
//             continue;
//         }
//
//         let mean = mean_angle(&directions);
//         map.insert(flock.id, mean);
//     }
//
//     for (_, mut flock, _, mut movement) in &mut query {
//         if let Some(angle) = map.get(&flock.id) {
//             let velocity = Vec2::from_angle(*angle) * movement.velocity.length();
//             flock.direction = velocity;
//             movement.target_velocity = Some(velocity);
//         } else {
//             flock.direction = Vec2::new(0., 0.);
//         }
//     }
// }
//
// fn mean_angle(directions: &[f64]) -> f32 {
//     if directions.is_empty() {
//         return 0.;
//     }
//
//     let len = directions.len() as f64;
//     let x = directions.iter().map(|x| x.cos()).sum::<f64>() / len;
//     let y = directions.iter().map(|y| y.sin()).sum::<f64>() / len;
//     y.atan2(x) as f32
// }

// fn boids_raycast_drawing_system(
//     config: ResMut<Configuration>,
//     mut gizmos: Gizmos,
//     mut query: Query<(&mut Transform, &CollisionVolume, Option<&mut BoidMovement>)>,
// ) {
//     let mut map: HashMap<usize, f32> = HashMap::new();
//     for (transform, coll_volume, movement) in &query {
//         if let Some(movement) = movement {
//             let center = transform.translation.xy();
//             let direction_angle = movement.velocity.to_angle();
//
//             if config.ray_debug {
//                 gizmos.arc_2d(
//                     center,
//                     PI / 2. - direction_angle,
//                     RAYCAST_FOV,
//                     RAYCAST_DIST,
//                     Color::FUCHSIA.with_a(0.2),
//                 );
//             }
//             for idx in 0..RAY_COUNT {
//                 let (hits, angle) = cast_ray_and_check(
//                     idx,
//                     &mut gizmos,
//                     center,
//                     direction_angle,
//                     coll_volume,
//                     &query,
//                     config.ray_debug,
//                 );
//
//                 if !hits {
//                     map.insert(coll_volume.id, angle);
//                     break;
//                 }
//             }
//             if map.get(&coll_volume.id).is_none() {
//                 map.insert(coll_volume.id, PI / 2.);
//             }
//         }
//     }
//
//     for (_, current_volume, movement) in &mut query {
//         if let Some(mut movement) = movement {
//             if let Some(angle) = map.get(&current_volume.id) {
//                 if *angle == movement.velocity.to_angle() {
//                     println!("boids_raycast_drawing_system early out");
//                     continue;
//                 }
//                 let velocity = Vec2::from_angle(*angle) * movement.velocity.length();
//                 movement.target_velocity = Some(velocity);
//             }
//         }
//     }
// }

// fn cast_ray_and_check(
//     idx: u8,
//     gizmos: &mut Gizmos,
//     center: Vec2,
//     direction_angle: f32,
//     coll_volume: &CollisionVolume,
//     volumes_query: &Query<(&mut Transform, &CollisionVolume, Option<&mut BoidMovement>)>,
//     debug: bool,
// ) -> (bool, f32) {
//     let ray_spacing = RAYCAST_FOV / (RAY_COUNT - 1) as f32;
//     // This whole thing cost me a day to figure out
//     let idx_even = idx % 2 == 0;
//     let div = (idx as i8 / 2) as f32;
//     let mul: f32 = if idx_even { -1. } else { 1. };
//     let ray_angle = direction_angle + mul * (idx as f32 * ray_spacing - (div * ray_spacing));
//     let ray_vec = Vec2::from_angle(ray_angle);
//     let ray_cast = RayCast2d::new(center, Direction2d::new(ray_vec).unwrap(), RAYCAST_DIST);
//
//     let mut hits = false;
//     for (transform, volume, _) in volumes_query.iter() {
//         if volume.id == coll_volume.id {
//             continue;
//         }
//         let aabb = volume.shape.aabb_2d(
//             transform.translation.xy(),
//             transform.rotation.to_euler(EulerRot::YXZ).2,
//         );
//         let coll = ray_cast.aabb_intersection_at(&aabb).is_some();
//         if coll {
//             hits = true;
//             break;
//         }
//     }
//     if debug {
//         gizmos.line_2d(
//             ray_cast.ray.origin,
//             ray_cast.ray.origin + ray_vec * ray_cast.max,
//             if hits {
//                 Color::CRIMSON
//             } else {
//                 Color::TURQUOISE
//             },
//         );
//     }
//
//     (hits, ray_angle - direction_angle)
// }

// #[cfg(test)]
// mod test {
//     use crate::mean_angle;
//
//     macro_rules! assert_diff {
//         ($x: expr,$y : expr, $diff :expr) => {
//             if ($x - $y).abs() > $diff {
//                 panic!("floating point difference is to big {}", ($x - $y));
//             }
//         };
//     }
//
//     #[test]
//     fn test_mean_angles_symmetric_three_1() {
//         let angles: Vec<_> = vec![
//             f64::to_radians(45.),
//             f64::to_radians(90.),
//             f64::to_radians(0.),
//         ];
//
//         assert_diff!(f64::to_radians(45.) as f32, mean_angle(&angles), 0.001);
//     }
//
//     #[test]
//     fn test_mean_angles_symmetric_three_2() {
//         let angles: Vec<_> = vec![
//             f64::to_radians(180.),
//             f64::to_radians(90.),
//             f64::to_radians(0.),
//         ];
//
//         assert_diff!(f32::to_radians(90.), mean_angle(&angles), 0.001);
//     }
//
//     #[test]
//     fn test_mean_angles_3() {
//         let angles: Vec<_> = vec![
//             f64::to_radians(90.),
//             f64::to_radians(180.),
//             f64::to_radians(270.),
//             f64::to_radians(360.),
//         ];
//         assert_diff!(f32::to_radians(-90.), mean_angle(&angles), 0.001);
//     }
//
//     #[test]
//     fn test_mean_angles_4() {
//         let angles: Vec<_> = vec![f64::to_radians(350.), f64::to_radians(10.)];
//         assert_diff!(f32::to_radians(0.), mean_angle(&angles), 0.001);
//     }
//
//     #[test]
//     fn calculate() {
//         let angles1 = [350.0_f64, 10.0].map(|x| x.to_radians());
//         let angles2 = [90.0_f64, 180.0, 270.0, 360.0].map(|x| x.to_radians());
//         let angles3 = [10.0_f64, 20.0, 30.0].map(|x| x.to_radians());
//         assert_diff!(0_f32.to_radians(), mean_angle(&angles1), 0.001);
//         assert_diff!(-90.0_f32.to_radians(), mean_angle(&angles2), 0.001);
//         assert_diff!(20.0_f32.to_radians(), mean_angle(&angles3), 0.001);
//     }
// }
