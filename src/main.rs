use bevy::{
    math::{primitives::Rectangle, vec2},
    prelude::*,
    reflect::Map,
    sprite::MaterialMesh2dBundle,
    utils::HashMap,
    window::{close_on_esc, WindowResolution},
};
use bevy_math::bounding::{Bounded2d, RayCast2d};

#[derive(Component)]
struct BoidEntity;

#[derive(Component, Debug)]
struct CurrentVolume {
    pub id: u8,
    pub shape: Rectangle,
}

#[derive(Component, Default)]
struct Movement {
    pub velocity: f32,
    /// direction in radians
    pub direction: f32,
    rotation_speed: f32,
}

impl Movement {
    fn new(velocity: f32, direction: f32) -> Self {
        Self {
            velocity,
            direction,
            rotation_speed: f32::to_radians(90.0),
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

#[derive(Component, Deref, DerefMut, Default)]
struct Intersects(bool);

/// global properties
const WINDOW_SIZE: Vec2 = vec2(1900_f32, 1200_f32);
const BOID_COUNT: u8 = 7;
const INTER_BOID_SPACING: f32 = 200.0;

/// boid spawn properties
const BOID_SIZE: f32 = 50.0;
const TOTAL_SIZE: f32 = BOID_COUNT as f32 * BOID_SIZE;
const TOTAL_SPACING: f32 = INTER_BOID_SPACING * (BOID_COUNT - 1) as f32;
const TOTAL_OFFSET: f32 = (TOTAL_SIZE + TOTAL_SPACING) / 2.0;
const BOID_VELOCITY: f32 = 150.;

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
const RAYCAST_FOV: f32 = 135. * (PI / 180_f32);
const RAY_COUNT: u8 = 21;
const RAYCAST_DIST: f32 = 200.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y),
                title: "Boids Demo".to_string(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                close_on_esc,
                boids_raycast_drawing_system,
                boids_movement_system,
            )
                .chain(),
        )
        .add_systems(Update, boids_update_volumes_system)
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
        Intersects::default(),
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
        Intersects::default(),
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
            id: 2,
            shape: Rectangle::new(HORIZONTAL_WALL_SIZE, WALL_THICKNESS),
        },
        Intersects::default(),
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
            id: 2,
            shape: Rectangle::new(WALL_THICKNESS, VERTICAL_WALL_SIZE),
        },
        Intersects::default(),
    ));

    let ship_handle = asset_server.load("textures/ship_C.png");
    (0..BOID_COUNT).for_each(|idx| {
        let idx = idx as f32;

        let direction_degrees = fastrand::f32() * 360.0;
        let direction_degrees = f32::to_radians(direction_degrees);
        commands.spawn((
            SpriteBundle {
                texture: ship_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    (idx * INTER_BOID_SPACING) - TOTAL_OFFSET,
                    -300.0,
                    0.0,
                )),
                ..default()
            },
            BoidEntity,
            CurrentVolume {
                id: idx as u8 * 10,
                shape: Rectangle::new(BOID_SIZE, BOID_SIZE),
            },
            Movement::new(BOID_VELOCITY, direction_degrees),
            Intersects::default(),
        ));
    })
}

fn boids_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Movement), With<BoidEntity>>,
) {
    // move boids forward the direction it's facing
    for (mut transform, movement) in &mut query {
        transform.rotation = Quat::from_rotation_z(movement.direction);
        let movement_direction = transform.rotation * Vec3::Y;
        let movement_distance = 1.0 * movement.velocity * time.delta_seconds();
        let translation_delta = movement_direction * movement_distance;
        transform.translation += translation_delta;
    }
}

fn cast_ray_and_check(
    idx: u8,
    gizmos: &mut Gizmos,
    center: Vec2,
    direction_angle: f32,
    ray_spacing: f32,
    current_volume: &CurrentVolume,
    volumes_query: &Query<(&mut Transform, &CurrentVolume, Option<&mut Movement>)>,
) -> (bool, f32) {
    let idx_even = idx % 2 == 0;
    let div = (idx as i8 / 2) as f32;
    let mul: f32 = if idx_even { -1. } else { 1. };
    let ray_angle = direction_angle + mul * (idx as f32 * ray_spacing - (div * ray_spacing));
    let ray_vec = Vec2::new(f32::cos(ray_angle), f32::sin(ray_angle));
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
    gizmos.line_2d(
        ray_cast.ray.origin,
        ray_cast.ray.origin + ray_vec * ray_cast.max,
        if hits {
            Color::CRIMSON
        } else {
            Color::TURQUOISE
        },
    );

    (hits, ray_angle - direction_angle)
}

fn boids_raycast_drawing_system(
    mut gizmos: Gizmos,
    mut query: Query<(&mut Transform, &CurrentVolume, Option<&mut Movement>)>,
) {
    let mut map: HashMap<u8, f32> = HashMap::new();
    for (transform, current_volume, movement) in &query {
        if let Some(movement) = movement {
            let center = transform.translation.xy();
            let direction_angle = movement.direction + PI / 2.;

            gizmos.arc_2d(
                center,
                PI / 2. - direction_angle,
                RAYCAST_FOV,
                RAYCAST_DIST,
                Color::FUCHSIA.with_a(0.2),
            );
            let ray_spacing = RAYCAST_FOV / (RAY_COUNT - 1) as f32;
            for idx in 0..RAY_COUNT {
                let (hits, angle) = cast_ray_and_check(
                    idx,
                    &mut gizmos,
                    center,
                    direction_angle,
                    ray_spacing,
                    current_volume,
                    &query,
                );

                if !hits {
                    map.insert(current_volume.id, angle);
                    break;
                }
            }
        }
    }

    for (_, current_volume, movement) in &mut query {
        if let Some(mut movement) = movement {
            if let Some(angle) = map.get(&current_volume.id) {
                movement.direction += angle;
            }
        }
    }
}

fn boids_update_volumes_system(
    // mut commands: Commands,
    mut gizmos: Gizmos,
    query: Query<(Entity, &CurrentVolume, &Transform)>,
) {
    for (_entity, volume, transform) in query.iter() {
        gizmos.rect_2d(
            transform.translation.xy(),
            transform.rotation.to_euler(EulerRot::YXZ).2,
            volume.shape.size(),
            Color::PINK,
        )
    }
}
