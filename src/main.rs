use bevy::{
    math::vec2,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::{close_on_esc, WindowResolution},
};
use bevy_math::bounding::RayCast2d;

#[derive(Component)]
struct BoidEntity;

#[derive(Component)]
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

/// global properties
const WINDOW_SIZE: Vec2 = vec2(1900_f32, 1200_f32);
const BOID_COUNT: u8 = 5;
const INTER_BOID_SPACING: f32 = 200.0;

/// boid spawn properties
const BOID_SIZE: f32 = 30.0;
const TOTAL_SIZE: f32 = BOID_COUNT as f32 * BOID_SIZE;
const TOTAL_SPACING: f32 = INTER_BOID_SPACING * (BOID_COUNT - 1) as f32;
const TOTAL_OFFSET: f32 = (TOTAL_SIZE + TOTAL_SPACING) / 2.0;

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
const RAY_COUNT: u8 = 12;
const RAYCAST_DIST: f32 = 150.;

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
        .add_systems(Update, (close_on_esc, boids_raycast_drawing_system))
        .add_systems(FixedUpdate, boids_movement_system)
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
    ));

    let ship_handle = asset_server.load("textures/ship_C.png");
    (0..BOID_COUNT).for_each(|idx| {
        let idx = idx as f32;

        let direction_degrees = fastrand::f32() * 360.0;
        let direction_degrees = f32::to_radians(direction_degrees);
        info!("deg: {}", direction_degrees);
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
            Movement::new(10.0, direction_degrees),
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

fn boids_raycast_drawing_system(
    mut gizmos: Gizmos,
    mut query: Query<(&mut Transform, &mut Movement), With<BoidEntity>>,
) {
    for (transform, movement) in &mut query {
        let center = transform.translation.xy();
        let direction_angle = movement.direction + PI / 2.;

        gizmos.arc_2d(
            center,
            PI / 2. - direction_angle,
            RAYCAST_FOV,
            RAYCAST_DIST,
            Color::FUCHSIA,
        );
        let ray_spacing = RAYCAST_FOV / (RAY_COUNT - 1) as f32;
        for idx in 0..RAY_COUNT {
            let ray_angle = direction_angle + RAYCAST_FOV / 2. - ray_spacing * idx as f32;
            let ray_vec = Vec2::new(f32::cos(ray_angle), f32::sin(ray_angle));
            let ray_cast = RayCast2d::new(center, Direction2d::new(ray_vec).unwrap(), RAYCAST_DIST);
            gizmos.line_2d(
                ray_cast.ray.origin,
                ray_cast.ray.origin + ray_vec * ray_cast.max,
                Color::CRIMSON,
            );
        }
    }
}
