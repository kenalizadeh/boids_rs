use crate::{BoidFlock, BoidMovement, CollisionVolume, GridRect, Wall};
use bevy::{math::primitives::Rectangle, prelude::*, sprite::MaterialMesh2dBundle};

/// global properties
pub const WINDOW_SIZE: Vec2 = Vec2::new(1900_f32, 1200_f32);
const BOID_COUNT: u8 = 64;

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

/// boid spawn properties
const BOID_SIZE: Vec2 = Vec2::new(20., 60.);
const BOID_SPEED: f32 = 50.;

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // WALLS
    for &(id, v_size, h_size, x_pos, y_pos) in walls_slice() {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(bevy_math::primitives::Rectangle::new(v_size, h_size))
                    .into(),
                material: materials.add(ColorMaterial::from(WALL_COLOR)),
                transform: Transform::from_translation(Vec3::new(x_pos, y_pos, WALL_Z)),
                ..default()
            },
            Wall,
            CollisionVolume::new(id, Rectangle::new(v_size, h_size)),
        ));
    }

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
            CollisionVolume::new(idx * 10, Rectangle::from_size(BOID_SIZE)),
            BoidMovement::new(
                Vec2::from_angle(direction_degrees) * BOID_SPEED,
                std::f32::consts::PI,
            ),
            BoidFlock::new(idx),
        ));
    }
}

fn grid_row_col(x: u32) -> u32 {
    ((x as f32).sqrt().ceil() as u32).max(2)
}

fn tile_window(tile_size: u32) -> Vec<GridRect> {
    let tile_size = grid_row_col(tile_size);
    let width: f32 = WINDOW_SIZE.x / tile_size as f32;
    let height: f32 = WINDOW_SIZE.y / tile_size as f32;

    let mut grids: Vec<GridRect> = vec![];
    for r in 0..tile_size {
        for c in 0..tile_size {
            grids.push(GridRect::new(
                r as f32 * width - (WINDOW_SIZE.x) / 2. + width / 2.,
                c as f32 * height - (WINDOW_SIZE.y) / 2. + height / 2.,
                width,
                height,
            ))
        }
    }

    grids
}

fn walls_slice() -> &'static [(usize, f32, f32, f32, f32)] {
    &[
        (
            1_usize,
            HORIZONTAL_WALL_SIZE,
            WALL_THICKNESS,
            0_f32,
            TOP_WALL_POS,
        ),
        (
            2_usize,
            WALL_THICKNESS,
            VERTICAL_WALL_SIZE,
            LEFT_WALL_POS,
            0_f32,
        ),
        (
            3_usize,
            HORIZONTAL_WALL_SIZE,
            WALL_THICKNESS,
            0_f32,
            BOTTOM_WALL_POS,
        ),
        (
            4_usize,
            WALL_THICKNESS,
            VERTICAL_WALL_SIZE,
            RIGHT_WALL_POS,
            0_f32,
        ),
    ]
}
