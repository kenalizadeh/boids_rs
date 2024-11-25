use bevy::{
    color::LinearRgba,
    ecs::system::{Query, Res},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::Time,
    transform::components::Transform,
    window::WindowResized,
};

// MOVEMENT
pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                boids_rotation_system,
                rule_velocity_comb_system,
                boids_forward_movement_system,
                boids_teleport_system,
            )
                .chain(),
        );
    }
}

#[derive(Component, Default)]
pub struct BoidMovement {
    pub id: usize,
    pub speed: f32,
    pub target_angle: f32,
    pub rotation_speed: f32,
}

impl BoidMovement {
    pub fn new(id: usize, speed: f32, target_angle: f32, rotation_speed: f32) -> Self {
        Self {
            id,
            speed,
            target_angle,
            rotation_speed,
        }
    }
}

// 2D Rotation Example helped a lot with this.
// https://bevyengine.org/examples/2D%20Rendering/rotation/
fn boids_rotation_system(time: Res<Time>, mut query: Query<(&mut Transform, &BoidMovement)>) {
    for (mut transform, movement) in &mut query {
        let curr_vel = (transform.rotation * Vec3::Y).xy();
        let target_vel = Vec2::from_angle(movement.target_angle).normalize();
        let target_vel_dot = curr_vel.normalize().dot(target_vel);

        if (target_vel_dot - 1.).abs() < f32::EPSILON {
            continue;
        }

        let right_vector = (transform.rotation * Vec3::X).xy();

        let right_vector_dot = right_vector.dot(target_vel);

        let rotation_sign = -f32::copysign(1.0, right_vector_dot);

        let max_angle = target_vel_dot.clamp(-1.0, 1.0).acos();

        let rotation_angle =
            rotation_sign * (movement.rotation_speed * time.delta_seconds()).min(max_angle);

        transform.rotate_z(rotation_angle);
    }
}

fn rule_velocity_comb_system(
    mut query: Query<(
        &mut BoidMovement,
        &SeparationRule,
        &AlignmentRule,
        &CohesionRule,
    )>,
) {
    for (mut movement, separation, alignment, cohesion) in &mut query {
        let velocities = [separation.velocity, alignment.velocity, cohesion.velocity];
        let velocity: Vec2 = velocities.iter().map(|v| v.normalize()).sum();

        if !velocity.is_nan() {
            movement.target_angle = velocity.to_angle();
        }
    }
}

fn boids_forward_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &BoidMovement), With<BoidMovement>>,
) {
    for (mut transform, movement) in &mut query {
        let movement_direction = transform.rotation * Vec3::Y;
        let movement_distance = movement.speed * time.delta_seconds();
        let translation_delta = movement_direction * movement_distance;
        transform.translation += translation_delta;
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

// RULES
pub struct RulesPlugin;

impl Plugin for RulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (separation_system, alignment_system, cohesion_system).chain(),
        );
    }
}

#[derive(Component)]
pub struct SeparationRule {
    pub id: usize,
    pub radius: f32,
    // between 0.0 and 1.0
    // 0 means off
    pub factor: f32,
    pub velocity: Vec2,
}

impl SeparationRule {
    pub fn new(id: usize, radius: f32, factor: f32, velocity: Vec2) -> Self {
        Self {
            id,
            radius,
            factor,
            velocity,
        }
    }
}

#[derive(Component)]
pub struct AlignmentRule {
    pub id: usize,
    pub radius: f32,
    // between 0.0 and 1.0
    // 0 means off
    pub factor: f32,
    pub velocity: Vec2,
}

impl AlignmentRule {
    pub fn new(id: usize, radius: f32, factor: f32, velocity: Vec2) -> Self {
        Self {
            id,
            radius,
            factor,
            velocity,
        }
    }
}

#[derive(Component)]
pub struct CohesionRule {
    pub id: usize,
    pub radius: f32,
    // between 0.0 and 1.0
    // 0 means off
    pub factor: f32,
    pub velocity: Vec2,
}

impl CohesionRule {
    pub fn new(id: usize, radius: f32, factor: f32, velocity: Vec2) -> Self {
        Self {
            id,
            radius,
            factor,
            velocity,
        }
    }
}

fn separation_system(
    mut gizmos: Gizmos,
    mut query: Query<(&Transform, &mut SeparationRule, &BoidMovement)>,
) {
    let mut velocities: [Option<Vec2>; BOID_COUNT] = [Option::None; BOID_COUNT];
    for (current_transform, current_separation, current_movement) in &query {
        let current_center = current_transform.translation.xy();
        let mut nearby_boid_count = 0_u8;
        let mut velocity = Vec2::ZERO;

        if current_movement.id == DEBUG_BOID_ID {
            gizmos.circle_2d(current_center, current_separation.radius, LinearRgba::RED);
            gizmos.arrow_2d(
                current_center,
                current_center + current_separation.velocity,
                LinearRgba::RED,
            );
        }

        for (transform, separation, _) in &query {
            if separation.id == current_separation.id {
                continue;
            }

            let center = transform.translation.xy();
            let distance = current_center.distance(center);
            if distance > current_separation.radius {
                continue;
            }

            let init_velocity = current_center - center;
            let weight = (current_separation.radius - distance) / current_separation.radius;
            let weighted_velocity = init_velocity.normalize() * weight * current_movement.speed;

            velocity += weighted_velocity;
            nearby_boid_count += 1;
        }

        if nearby_boid_count > 0 {
            velocity /= nearby_boid_count as f32;
            velocity *= current_separation.factor;

            velocities[current_separation.id] = Some(velocity);
        }
    }

    for (_, mut separation, _) in &mut query {
        let vel = velocities[separation.id].unwrap_or(Vec2::ZERO);
        separation.velocity = vel;
    }
}

fn alignment_system(
    mut gizmos: Gizmos,
    mut query: Query<(&Transform, &mut AlignmentRule, &BoidMovement)>,
) {
    let mut velocities: [Option<Vec2>; BOID_COUNT] = [Option::None; BOID_COUNT];
    for (current_transform, current_alignment, current_movement) in &query {
        let current_center = current_transform.translation.xy();
        let mut nearby_boid_count = 0_u8;
        let mut velocity = Vec2::ZERO;

        if current_movement.id == DEBUG_BOID_ID {
            gizmos.circle_2d(current_center, current_alignment.radius, LinearRgba::GREEN);
            gizmos.arrow_2d(
                current_center,
                current_center + current_alignment.velocity,
                LinearRgba::GREEN,
            );
        }

        for (transform, alignment, _) in &query {
            // skip over current boid
            if alignment.id == current_alignment.id {
                continue;
            }

            // filter out out-of-reach boids
            let center = transform.translation.xy();
            let distance = current_center.distance(center);
            if distance > current_alignment.radius {
                continue;
            }

            let init_velocity = (transform.rotation * Vec3::Y).xy();
            let weight = (current_alignment.radius - distance) / current_alignment.radius;
            let weighted_velocity = init_velocity.normalize() * weight * current_movement.speed;

            velocity += weighted_velocity;
            nearby_boid_count += 1;

            if current_movement.id == DEBUG_BOID_ID {
                gizmos.line_2d(current_center, center, LinearRgba::BLUE);
            }
        }

        if nearby_boid_count > 0 {
            velocity /= nearby_boid_count as f32;
            velocity *= current_alignment.factor;

            velocities[current_alignment.id] = Some(velocity);
        }
    }

    for (_, mut alignment, _) in &mut query {
        let vel = velocities[alignment.id].unwrap_or(Vec2::ZERO);
        alignment.velocity = vel;
    }
}

fn cohesion_system(
    mut gizmos: Gizmos,
    mut query: Query<(&Transform, &mut CohesionRule, &BoidMovement)>,
) {
    let mut velocities: [Option<Vec2>; BOID_COUNT] = [Option::None; BOID_COUNT];
    for (current_transform, current_cohesion, current_movement) in &query {
        let current_center = current_transform.translation.xy();
        let mut nearby_boid_count = 0_u8;
        let mut center_of_mass = current_center;
        let mut boid_positions: Vec<Vec2> = vec![];

        if current_movement.id == DEBUG_BOID_ID {
            gizmos.circle_2d(current_center, current_cohesion.radius, LinearRgba::WHITE);
            gizmos.arrow_2d(
                current_center,
                current_center + current_cohesion.velocity,
                LinearRgba::WHITE,
            );
        }

        for (transform, cohesion, _) in &query {
            if cohesion.id == current_cohesion.id {
                continue;
            }

            let center = transform.translation.xy();
            let distance = current_center.distance(center);
            if distance > current_cohesion.radius {
                continue;
            }

            center_of_mass += center;
            nearby_boid_count += 1;

            boid_positions.push(center);
        }

        if nearby_boid_count > 0 {
            center_of_mass -= current_center;
            center_of_mass /= nearby_boid_count as f32;

            let com_vector = center_of_mass - current_center;
            let weight = (current_cohesion.radius - com_vector.length()) / current_cohesion.radius;
            let weighted_velocity =
                com_vector.normalize() * weight * current_movement.speed * current_cohesion.factor;

            velocities[current_cohesion.id] = Some(weighted_velocity);
        }
    }

    for (_, mut cohesion, _) in &mut query {
        let vel = velocities[cohesion.id].unwrap_or(Vec2::ZERO);
        cohesion.velocity = vel;
    }
}

// STARTUP
// global properties
pub const INITIAL_WINDOW_SIZE: Vec2 = Vec2::new(2560_f32, 1800_f32);
pub const BOID_COUNT: usize = 128;
pub const DEBUG_BOID_ID: usize = BOID_COUNT + 1;

// Walls
const WALL_THICKNESS: f32 = 10.0;
const WALL_Z: f32 = 10.0;
const WALL_COLOR: LinearRgba = LinearRgba::GREEN;

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, window_walls_resize_system);
    }
}

#[derive(Debug)]
pub struct RectFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectFrame {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            width: w,
            height: h,
        }
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn size(&self) -> Rectangle {
        Rectangle::new(self.width, self.height)
    }
}

#[derive(Component)]
pub enum Wall {
    Top,
    Right,
    Bottom,
    Left,
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
                material: materials.add(ColorMaterial::from_color(WALL_COLOR)),
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
                transform: Transform::from_xyz(grid.x, grid.y, idx as f32)
                    .with_rotation(Quat::from_rotation_z(direction_degrees)),
                ..default()
            },
            SeparationRule::new(idx, 175., 1., Vec2::ZERO),
            AlignmentRule::new(idx, 100., 1., Vec2::ZERO),
            CohesionRule::new(idx, 200., 1., Vec2::ZERO),
            BoidMovement::new(idx, 150., target_degrees, std::f32::consts::PI / 2.),
        ));
    }
}

fn make_random_pastel_color() -> Color {
    const LIGHT_BLUE_R: f32 = 173. / 255.;
    const LIGHT_BLUE_G: f32 = 216. / 255.;
    const LIGHT_BLUE_B: f32 = 230. / 255.;

    Color::srgb(
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
