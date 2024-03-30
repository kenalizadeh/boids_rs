use super::components::RectFrame;
use crate::plugins::components::{AlignmentRule, BoidMovement, CohesionRule, SeparationRule, Wall};
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResized,
};

// global properties
pub const INITIAL_WINDOW_SIZE: Vec2 = Vec2::new(2560_f32, 1800_f32);
pub const BOID_COUNT: usize = 128;

// Walls
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
    for (wall, frame) in [
        (Wall::Top, get_top_wall_frame(INITIAL_WINDOW_SIZE)),
        (Wall::Left, get_left_wall_frame(INITIAL_WINDOW_SIZE)),
        (Wall::Bottom, get_bottom_wall_frame(INITIAL_WINDOW_SIZE)),
        (Wall::Right, get_right_wall_frame(INITIAL_WINDOW_SIZE)),
    ] {
        let pos = frame.pos();
        let size = frame.size();

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(size).into(),
                material: materials.add(ColorMaterial::from(WALL_COLOR)),
                transform: Transform::from_translation(Vec3::new(pos.x, pos.y, WALL_Z)),
                ..default()
            },
            wall,
        ));
    }

    let grids_vec = tile_window(BOID_COUNT as u32);
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
            SeparationRule::new(idx, 100., 1., Vec2::ZERO),
            AlignmentRule::new(idx, 100., 1., Vec2::ZERO),
            CohesionRule::new(idx, 100., 1., Vec2::ZERO),
            BoidMovement::new(90., target_degrees, std::f32::consts::PI),
        ));
    }
}

fn make_random_pastel_color() -> Color {
    const LIGHT_BLUE_R: f32 = 173. / 255.;
    const LIGHT_BLUE_G: f32 = 216. / 255.;
    const LIGHT_BLUE_B: f32 = 230. / 255.;

    Color::rgb(
        (fastrand::f32() + LIGHT_BLUE_R) / 2.,
        (fastrand::f32() + LIGHT_BLUE_G) / 2.,
        (fastrand::f32() + LIGHT_BLUE_B) / 2.,
    )
}

/// get min square number for given boid count
/// e.g. 25 if boid count is between 16 and 25
fn grid_row_col(x: u32) -> u32 {
    ((x as f32).sqrt().ceil() as u32).max(2)
}

fn tile_window(tile_size: u32) -> Vec<RectFrame> {
    let tile_size = grid_row_col(tile_size);
    let width: f32 = INITIAL_WINDOW_SIZE.x / tile_size as f32;
    let height: f32 = INITIAL_WINDOW_SIZE.y / tile_size as f32;

    let mut grids: Vec<RectFrame> = vec![];
    for r in 0..tile_size {
        for c in 0..tile_size {
            grids.push(RectFrame::new(
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
    mut wall_query: Query<(&mut Transform, &mut Mesh2dHandle, &Wall)>,
    mut resize_reader: EventReader<WindowResized>,
) {
    if let Some(window) = resize_reader.read().next() {
        let res = Vec2::new(window.width, window.height);

        for (mut transform, mut mesh, wall) in &mut wall_query {
            let frame = match &wall {
                Wall::Top => get_top_wall_frame(res),
                Wall::Left => get_left_wall_frame(res),
                Wall::Bottom => get_bottom_wall_frame(res),
                Wall::Right => get_right_wall_frame(res),
            };

            *mesh = meshes.add(frame.size()).into();
            let pos = frame.pos();
            transform.translation = Vec3::new(pos.x, pos.y, WALL_Z);
        }
    }
}

fn get_top_wall_frame(res: Vec2) -> RectFrame {
    RectFrame::new(0., (res.y - WALL_THICKNESS) / 2., res.x, WALL_THICKNESS)
}

fn get_left_wall_frame(res: Vec2) -> RectFrame {
    RectFrame::new((-res.x + WALL_THICKNESS) / 2., 0., WALL_THICKNESS, res.y)
}

fn get_bottom_wall_frame(res: Vec2) -> RectFrame {
    RectFrame::new(0., (-res.y + WALL_THICKNESS) / 2., res.x, WALL_THICKNESS)
}

fn get_right_wall_frame(res: Vec2) -> RectFrame {
    RectFrame::new((res.x - WALL_THICKNESS) / 2., 0., WALL_THICKNESS, res.y)
}
