use crate::{
    AlignmentRule, BoidMovement, CohesionRule, CollisionVolume, GridRect, SeparationRule, Wall,
};
use bevy::{
    math::primitives::Rectangle,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResized,
};
use core::panic;

/// global properties
pub const INITIAL_WINDOW_SIZE: Vec2 = Vec2::new(2560_f32, 1800_f32);
const BOID_COUNT: usize = 64;

/// Walls
const WALL_ID_OFFSET: usize = BOID_COUNT + 10;
const TOP_WALL_ID: usize = WALL_ID_OFFSET + 1;
const LEFT_WALL_ID: usize = WALL_ID_OFFSET + 2;
const BOTTOM_WALL_ID: usize = WALL_ID_OFFSET + 3;
const RIGHT_WALL_ID: usize = WALL_ID_OFFSET + 4;
const WALL_THICKNESS: f32 = 10.0;
const WALL_Z: f32 = 10.0;
const WALL_COLOR: Color = Color::DARK_GREEN;

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, window_walls_resize_system);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // WALLS
    for &(id, (pos, size)) in &[
        (TOP_WALL_ID, get_top_wall_rect(INITIAL_WINDOW_SIZE)),
        (LEFT_WALL_ID, get_left_wall_rect(INITIAL_WINDOW_SIZE)),
        (BOTTOM_WALL_ID, get_bottom_wall_rect(INITIAL_WINDOW_SIZE)),
        (RIGHT_WALL_ID, get_right_wall_rect(INITIAL_WINDOW_SIZE)),
    ] {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(size).into(),
                material: materials.add(ColorMaterial::from(WALL_COLOR)),
                transform: Transform::from_translation(Vec3::new(pos.x, pos.y, WALL_Z)),
                ..default()
            },
            Wall::new(size),
            CollisionVolume::new(id, size),
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

    // let boid_handle = asset_server.load("textures/berd.png");
    assert!(grids_vec.len() >= BOID_COUNT);
    for (idx, grid) in grids_vec.iter().take(BOID_COUNT).enumerate() {
        let direction_degrees = (fastrand::f32() * 360.0).to_radians();
        let target_degrees = (fastrand::f32() * 360.0).to_radians();
        let rand_color = make_random_pastel_color();

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(RegularPolygon::new(20., 3)).into(),
                material: materials.add(ColorMaterial::from(rand_color)),
                transform: Transform::from_xyz(grid.x, grid.y, 0.0)
                    .with_rotation(Quat::from_rotation_z(direction_degrees)),
                ..default()
            },
            // SpriteBundle {
            //     texture: boid_handle.clone(),
            //     transform: Transform::from_xyz(grid.x, grid.y, 0.0)
            //         .with_rotation(Quat::from_rotation_z(direction_degrees)),
            //     ..default()
            // },
            SeparationRule::new(idx, 100., 1., Vec2::ZERO),
            AlignmentRule::new(idx, 100., 1., Vec2::ZERO),
            CohesionRule::new(idx, 100., 1., Vec2::ZERO),
            BoidMovement::new(90., target_degrees, std::f32::consts::PI),
        ));
    }
}

fn make_random_pastel_color() -> Color {
    const LIGHT_BLUE: Color = Color::rgb(173. / 255., 216. / 255., 230. / 255.);

    Color::rgb(
        (fastrand::f32() + LIGHT_BLUE.r()) / 2.,
        (fastrand::f32() + LIGHT_BLUE.g()) / 2.,
        (fastrand::f32() + LIGHT_BLUE.b()) / 2.,
    )
}

fn grid_row_col(x: u32) -> u32 {
    ((x as f32).sqrt().ceil() as u32).max(2)
}

fn tile_window(tile_size: u32) -> Vec<GridRect> {
    let tile_size = grid_row_col(tile_size);
    let width: f32 = INITIAL_WINDOW_SIZE.x / tile_size as f32;
    let height: f32 = INITIAL_WINDOW_SIZE.y / tile_size as f32;

    let mut grids: Vec<GridRect> = vec![];
    for r in 0..tile_size {
        for c in 0..tile_size {
            grids.push(GridRect::new(
                r as f32 * width - (INITIAL_WINDOW_SIZE.x) / 2. + width / 2.,
                c as f32 * height - (INITIAL_WINDOW_SIZE.y) / 2. + height / 2.,
                width,
                height,
            ))
        }
    }

    grids
}

fn window_walls_resize_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut wall_query: Query<(
        &mut Transform,
        &mut Mesh2dHandle,
        &mut Wall,
        &mut CollisionVolume,
    )>,
    mut resize_reader: EventReader<WindowResized>,
) {
    if let Some(window) = resize_reader.read().next() {
        let res = Vec2::new(window.width, window.height);

        for (mut transform, mut mesh, mut wall, mut coll_volume) in &mut wall_query {
            let (pos, rect) = match coll_volume.id {
                TOP_WALL_ID => get_top_wall_rect(res),
                LEFT_WALL_ID => get_left_wall_rect(res),
                BOTTOM_WALL_ID => get_bottom_wall_rect(res),
                RIGHT_WALL_ID => get_right_wall_rect(res),
                _ => panic!("wall id not found"),
            };

            *mesh = meshes.add(rect).into();
            transform.translation = Vec3::new(pos.x, pos.y, WALL_Z);
            wall.rect = rect;
            coll_volume.shape = rect;
        }
    }
}

fn get_top_wall_rect(res: Vec2) -> (Vec2, Rectangle) {
    (
        Vec2::new(0., res.y / 2.0 - WALL_THICKNESS),
        Rectangle::new(res.x - WALL_THICKNESS, WALL_THICKNESS),
    )
}

fn get_left_wall_rect(res: Vec2) -> (Vec2, Rectangle) {
    (
        Vec2::new(-res.x / 2. + WALL_THICKNESS, 0.),
        Rectangle::new(WALL_THICKNESS, res.y - WALL_THICKNESS),
    )
}

fn get_bottom_wall_rect(res: Vec2) -> (Vec2, Rectangle) {
    (
        Vec2::new(0., -res.y / 2.0 + WALL_THICKNESS),
        Rectangle::new(res.x - WALL_THICKNESS, WALL_THICKNESS),
    )
}

fn get_right_wall_rect(res: Vec2) -> (Vec2, Rectangle) {
    (
        Vec2::new(res.x / 2. - WALL_THICKNESS, 0.),
        Rectangle::new(WALL_THICKNESS, res.y - WALL_THICKNESS),
    )
}
