use bevy::{
    math::{primitives::Rectangle, vec2},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
    window::{close_on_esc, WindowResolution},
};
use bevy_math::bounding::{Bounded2d, RayCast2d};

#[derive(Component)]
struct BoidEntity;

#[derive(Component)]
struct CurrentVolume {
    pub id: usize,
    pub shape: Rectangle,
}

#[derive(Component)]
struct BoidFlock {
    pub radius: f32,
    pub direction: Vec2,
}

impl BoidFlock {
    fn new() -> Self {
        BoidFlock {
            radius: 150.,
            direction: Vec2::new(0., 0.),
        }
    }
}

#[derive(Component, Default)]
struct Movement {
    pub velocity: Vec2,
    pub target_velocity: Option<Vec2>,
    pub rotation_speed: f32,
}

impl Movement {
    fn new(velocity: Vec2, rotation_speed: f32) -> Self {
        Self {
            velocity,
            target_velocity: Option::None,
            rotation_speed,
        }
    }
}

#[derive(Component)]
struct TopWall;

#[derive(Component)]
struct LeftWall;

#[derive(Component)]
struct RightWall;

#[derive(Component)]
struct BottomWall;

#[derive(Resource, Default)]
struct Configuration {
    pub debug: bool,
}

/// global properties
const WINDOW_SIZE: Vec2 = vec2(1900_f32, 1200_f32);
const BOID_COUNT: u8 = 64;
const INTER_BOID_SPACING: f32 = 200.0;

/// boid spawn properties
const BOID_SIZE: Vec2 = Vec2::new(20., 60.);
const TOTAL_SIZE: f32 = BOID_COUNT as f32 * BOID_SIZE.x;
const TOTAL_SPACING: f32 = INTER_BOID_SPACING * (BOID_COUNT - 1) as f32;
const TOTAL_OFFSET: f32 = (TOTAL_SIZE + TOTAL_SPACING) / 2.0;
const BOID_SPEED: f32 = 150.;

/// Walls
const WALL_THICKNESS: f32 = 10.0;
const TOP_WALL_POS: f32 = WINDOW_SIZE.y / 2.0 - WALL_THICKNESS;
const LEFT_WALL_POS: f32 = -WINDOW_SIZE.x / 2.0 + WALL_THICKNESS;
const RIGHT_WALL_POS: f32 = WINDOW_SIZE.x / 2.0 - WALL_THICKNESS;
const BOTTOM_WALL_POS: f32 = -WINDOW_SIZE.y / 2.0 + WALL_THICKNESS;
const WALL_Z: f32 = 10.0;

const HORIZONTAL_WALL_SIZE: f32 = RIGHT_WALL_POS - LEFT_WALL_POS + WALL_THICKNESS;
const VERTICAL_WALL_SIZE: f32 = TOP_WALL_POS - BOTTOM_WALL_POS + WALL_THICKNESS;
const WALL_COLOR: Color = Color::DARK_GREEN;

/// Raycast
const PI: f32 = std::f32::consts::PI;
const RAYCAST_FOV: f32 = 270. * (PI / 180_f32);
const RAY_COUNT: u8 = 31;
const RAYCAST_DIST: f32 = 200.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y),
                title: "Boids Demo".to_string(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(60.))
        .insert_resource(Configuration::default())
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                close_on_esc,
                boids_raycast_drawing_system,
                boids_movement_system,
                update_local_flock_direction,
            )
                .chain(),
        )
        .add_systems(Update, (boids_update_volumes_system, config_update_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // WALLS
    // top wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(bevy_math::primitives::Rectangle::new(
                    HORIZONTAL_WALL_SIZE,
                    WALL_THICKNESS,
                ))
                .into(),
            material: materials.add(ColorMaterial::from(WALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(0.0, TOP_WALL_POS, WALL_Z)),
            ..default()
        },
        TopWall,
        CurrentVolume {
            id: 1,
            shape: Rectangle::new(HORIZONTAL_WALL_SIZE, WALL_THICKNESS),
        },
    ));

    // left wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(bevy_math::primitives::Rectangle::new(
                    WALL_THICKNESS,
                    VERTICAL_WALL_SIZE,
                ))
                .into(),
            material: materials.add(ColorMaterial::from(WALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(LEFT_WALL_POS, 0.0, WALL_Z)),
            ..default()
        },
        LeftWall,
        CurrentVolume {
            id: 2,
            shape: Rectangle::new(WALL_THICKNESS, VERTICAL_WALL_SIZE),
        },
    ));

    // bottom wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(bevy_math::primitives::Rectangle::new(
                    HORIZONTAL_WALL_SIZE,
                    WALL_THICKNESS,
                ))
                .into(),
            material: materials.add(ColorMaterial::from(WALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(0.0, BOTTOM_WALL_POS, WALL_Z)),
            ..default()
        },
        BottomWall,
        CurrentVolume {
            id: 3,
            shape: Rectangle::new(HORIZONTAL_WALL_SIZE, WALL_THICKNESS),
        },
    ));

    // right wall
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(bevy_math::primitives::Rectangle::new(
                    WALL_THICKNESS,
                    VERTICAL_WALL_SIZE,
                ))
                .into(),
            material: materials.add(ColorMaterial::from(WALL_COLOR)),
            transform: Transform::from_translation(Vec3::new(RIGHT_WALL_POS, 0.0, WALL_Z)),
            ..default()
        },
        RightWall,
        CurrentVolume {
            id: 4,
            shape: Rectangle::new(WALL_THICKNESS, VERTICAL_WALL_SIZE),
        },
    ));

    let grids_vec = tile_window(BOID_COUNT as u32);
    // for grid in grids_vec {
    //     commands.spawn(MaterialMesh2dBundle {
    //         mesh: meshes.add(Rectangle::new(grid.width, grid.height)).into(),
    //         material: materials.add(ColorMaterial::from(Color::rgb(
    //             fastrand::f32(),
    //             fastrand::f32(),
    //             fastrand::f32(),
    //         ))),
    //         transform: Transform::from_xyz(grid.x, grid.y, 0.),
    //         ..default()
    //     });
    // }

    let ship_handle = asset_server.load("textures/berd.png");
    for (idx, grid) in grids_vec.iter().enumerate() {
        let direction_degrees = fastrand::f32() * 360.0;
        let direction_degrees = direction_degrees.to_radians();
        commands.spawn((
            SpriteBundle {
                texture: ship_handle.clone(),
                transform: Transform::from_xyz(grid.x, grid.y, 0.0)
                    .with_rotation(Quat::from_rotation_z(direction_degrees)),
                ..default()
            },
            BoidEntity,
            CurrentVolume {
                id: idx * 10,
                shape: Rectangle::from_size(BOID_SIZE),
            },
            Movement::new(Vec2::from_angle(direction_degrees) * BOID_SPEED, PI * 2.),
            BoidFlock::new(),
        ));
    }
}

fn grid_row_col(x: u32) -> u32 {
    ((x as f32).sqrt().ceil() as u32).max(2)
}

#[derive(Debug)]
struct GridRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl GridRect {
    fn center(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

fn tile_window(tile_size: u32) -> Vec<GridRect> {
    let tile_size = grid_row_col(tile_size);
    let width: f32 = WINDOW_SIZE.x / tile_size as f32;
    let height: f32 = WINDOW_SIZE.y / tile_size as f32;

    let mut grids: Vec<GridRect> = vec![];
    for r in 0..tile_size {
        for c in 0..tile_size {
            grids.push(GridRect {
                x: r as f32 * width - (WINDOW_SIZE.x) / 2. + width / 2.,
                y: c as f32 * height - (WINDOW_SIZE.y) / 2. + height / 2.,
                width,
                height,
            })
        }
    }

    grids
}

fn boids_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Movement), With<BoidEntity>>,
) {
    // move boids forward the direction it's facing
    for (mut transform, mut movement) in &mut query {
        if let Some(velocity) = movement.target_velocity {
            let angle = velocity.to_angle() * movement.rotation_speed * time.delta_seconds();
            let boid_fowrad_vec = (transform.rotation * Vec3::Y).xy();
            if (boid_fowrad_vec.to_angle() - angle).abs() < f32::EPSILON {
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

fn cast_ray_and_check(
    idx: u8,
    gizmos: &mut Gizmos,
    center: Vec2,
    direction_angle: f32,
    current_volume: &CurrentVolume,
    volumes_query: &Query<(&mut Transform, &CurrentVolume, Option<&mut Movement>)>,
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
        if volume.id == current_volume.id {
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

fn boids_raycast_drawing_system(
    config: ResMut<Configuration>,
    mut gizmos: Gizmos,
    mut query: Query<(&mut Transform, &CurrentVolume, Option<&mut Movement>)>,
) {
    let mut map: HashMap<usize, f32> = HashMap::new();
    for (transform, current_volume, movement) in &query {
        if let Some(movement) = movement {
            let center = transform.translation.xy();
            let direction_angle = movement.velocity.to_angle();

            if config.debug {
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
                    config.debug,
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

fn boids_update_volumes_system(
    config: ResMut<Configuration>,
    mut gizmos: Gizmos,
    query: Query<(Entity, &CurrentVolume, &Transform)>,
) {
    if !config.debug {
        return;
    }
    for (_entity, volume, transform) in query.iter() {
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
        config.debug = !config.debug;
    }
}
