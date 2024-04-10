use bevy::{
    app::{App, Startup, Update},
    asset::Assets,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    input::{keyboard::KeyCode, ButtonInput},
    math::{primitives::Circle, Vec3},
    prelude::*,
    render::{camera::Camera, color::Color, mesh::Mesh},
    sprite::{ColorMaterial, MaterialMesh2dBundle},
    transform::components::{GlobalTransform, Transform},
    utils::default,
    window::{close_on_esc, Window},
    DefaultPlugins,
};

use boids_rs::{AlignmentRule, BoidMovement, CohesionRule, SeparationRule};

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum RuleState {
    #[default]
    Separation,
    Alignment,
    Cohesion,
    Combined,
}

#[derive(Resource)]
struct Cursor {
    pos: Vec2,
}

fn separation_enabled(state: Res<State<RuleState>>) -> bool {
    matches!(**state, RuleState::Separation | RuleState::Combined)
}

fn alignment_enabled(state: Res<State<RuleState>>) -> bool {
    matches!(**state, RuleState::Alignment | RuleState::Combined)
}

fn cohesion_enabled(state: Res<State<RuleState>>) -> bool {
    matches!(**state, RuleState::Cohesion | RuleState::Combined)
}

fn all_rules_enabled(state: Res<State<RuleState>>) -> bool {
    matches!(**state, RuleState::Combined)
}

const RULES_RADIUS: f32 = 250.;
const TARGET_BOID_ID: usize = 0;

#[derive(Component)]
struct NearbyBoid;

#[derive(Component)]
struct ControlsText;

#[derive(Component)]
struct VelocityDebugText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .insert_resource(Cursor {
            pos: Vec2::new(0., 0.),
        })
        .init_state::<RuleState>()
        .add_systems(Update, state_change_system)
        .add_systems(
            PostUpdate,
            (
                cursor_system,
                cursor_gizmo_system,
                radius_gizmo_system,
                state_text_system,
                velocity_debug_text_system,
                clear_objects_system,
                object_spawn_system,
                separation_system.run_if(separation_enabled),
                alignment_system.run_if(alignment_enabled),
                cohesion_system.run_if(cohesion_enabled),
                combined_rules_system.run_if(all_rules_enabled),
                close_on_esc,
            )
                .chain(),
        )
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 26.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ControlsText,
    ));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 26.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        VelocityDebugText,
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(3.)).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        },
        SeparationRule::new(TARGET_BOID_ID, RULES_RADIUS, 1., Vec2::ZERO),
        AlignmentRule::new(TARGET_BOID_ID, RULES_RADIUS, 1., Vec2::ZERO),
        CohesionRule::new(TARGET_BOID_ID, RULES_RADIUS, 1., Vec2::ZERO),
        BoidMovement::new(90., 0., std::f32::consts::PI),
    ));
}

fn cursor_system(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut cursor: ResMut<Cursor>,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_pos) = window_query.single().cursor_position() else {
        return;
    };

    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    cursor.pos = point;
}

fn cursor_gizmo_system(mut gizmos: Gizmos, cursor: Res<Cursor>) {
    gizmos.circle_2d(cursor.pos, 5., Color::ANTIQUE_WHITE);
}

fn radius_gizmo_system(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &SeparationRule)>) {
    let (transform, separation) = query.single();
    let target_center = transform.translation().xy();

    gizmos.circle_2d(target_center, separation.radius, Color::CYAN);
}

fn clear_objects_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    objects_query: Query<Entity, With<NearbyBoid>>,
) {
    if key_input.just_pressed(KeyCode::KeyC) {
        for entity in &objects_query {
            commands.entity(entity).despawn()
        }
    }
}

fn object_spawn_system(
    key_input: Res<ButtonInput<KeyCode>>,
    cursor: Res<Cursor>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if key_input.just_pressed(KeyCode::KeyG) {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(5.)).into(),
                material: materials.add(ColorMaterial::from(Color::SEA_GREEN)),
                transform: Transform::from_translation(Vec3::new(cursor.pos.x, cursor.pos.y, 0.)),
                ..default()
            },
            BoidMovement::new(90., fastrand::f32() * 360., std::f32::consts::PI),
            NearbyBoid,
        ));
    }
}

fn state_text_system(
    mut query: Query<&mut Text, With<ControlsText>>,
    state: Res<State<RuleState>>,
) {
    if state.is_changed() {
        return;
    }

    let mut text = query.single_mut();
    let text = &mut text.sections[0].value;
    text.clear();

    text.push_str("Press (G) to add a boid\nPress (C) to clear");
    text.push_str("Rules:\n");
    for &st in &[
        RuleState::Separation,
        RuleState::Alignment,
        RuleState::Cohesion,
        RuleState::Combined,
    ] {
        let s = if **state == st { ">" } else { " " };
        text.push_str(&format!("{s} {st:?}\n"));
    }
    text.push_str("\npress Space to cycle");
}

fn velocity_debug_text_system(
    mut query: Query<&mut Text, With<VelocityDebugText>>,
    target: Query<(&mut SeparationRule, &mut AlignmentRule, &mut CohesionRule)>,
) {
    let (separation, alignment, cohesion) = target.single();
    let mut text = query.single_mut();
    let text = &mut text.sections[0].value;

    text.clear();
    text.push_str(&format!("Separation V: {}\n", separation.velocity));
    text.push_str(&format!("Alignment V: {}\n", alignment.velocity));
    text.push_str(&format!("Cohesion V: {}\n", cohesion.velocity));
}

fn state_change_system(
    key_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<RuleState>>,
    mut next_state: ResMut<NextState<RuleState>>,
) {
    if key_input.just_pressed(KeyCode::Space) {
        let next: RuleState = match **current_state {
            RuleState::Separation => RuleState::Alignment,
            RuleState::Alignment => RuleState::Cohesion,
            RuleState::Cohesion => RuleState::Combined,
            RuleState::Combined => RuleState::Separation,
        };

        next_state.set(next);
    }
}

fn separation_system(
    mut gizmos: Gizmos,
    mut target: Query<(&GlobalTransform, &mut SeparationRule)>,
    query: Query<(&Transform, &BoidMovement), With<NearbyBoid>>,
) {
    let (target, mut separation) = target.single_mut();
    let target_center = target.translation().xy();

    let mut nearby_boid_count = 0_f32;
    let mut velocity = Vec2::ZERO;

    for (transform, movement) in &query {
        let center = transform.translation.xy();
        let distance = target_center.distance(center);
        if distance > separation.radius {
            continue;
        }

        let forward_velocity = Vec2::from_angle(movement.target_angle) * movement.speed;
        gizmos.arrow_2d(center, center + forward_velocity, Color::LIME_GREEN);
        gizmos.line_2d(target_center, center, Color::DARK_GREEN);

        // adding vectors gives us the attraction velocity, subtracting does the opposite.
        let separation_velocity = target_center - center;
        let weight = (separation.radius - distance) / separation.radius;
        let weighted_velocity = separation_velocity.normalize() * weight;
        let final_velocity = weighted_velocity * movement.speed;

        velocity += final_velocity;
        nearby_boid_count += 1.;
    }

    if nearby_boid_count > 0. {
        velocity /= nearby_boid_count;
        velocity *= separation.factor;

        separation.velocity = velocity;

        gizmos.arrow_2d(target_center, target_center + velocity, Color::BLUE);
    } else {
        separation.velocity = Vec2::ZERO;
    }
}

fn alignment_system(
    mut gizmos: Gizmos,
    mut target: Query<(&GlobalTransform, &mut AlignmentRule)>,
    query: Query<(&Transform, &BoidMovement), With<NearbyBoid>>,
) {
    let (target, mut alignment) = target.single_mut();
    let target_center = target.translation().xy();

    let mut nearby_boid_count = 0_f32;
    let mut velocity = Vec2::ZERO;

    for (transform, movement) in &query {
        let center = transform.translation.xy();
        let distance = target_center.distance(center);
        if distance > alignment.radius {
            continue;
        }

        let forward_velocity = Vec2::from_angle(movement.target_angle) * movement.speed;
        gizmos.arrow_2d(center, center + forward_velocity, Color::LIME_GREEN);
        gizmos.line_2d(target_center, center, Color::DARK_GREEN);

        velocity += forward_velocity;
        nearby_boid_count += 1.;
    }

    if nearby_boid_count > 0. {
        velocity /= nearby_boid_count;
        velocity *= alignment.factor;

        alignment.velocity = velocity;

        gizmos.arrow_2d(
            target_center,
            target_center + velocity.normalize() * 50.,
            Color::BLUE,
        );
    } else {
        alignment.velocity = Vec2::ZERO;
    }
}

fn cohesion_system(
    mut gizmos: Gizmos,
    mut target: Query<(&GlobalTransform, &mut CohesionRule)>,
    query: Query<(&Transform, &BoidMovement), With<NearbyBoid>>,
) {
    let (target, mut cohesion) = target.single_mut();
    let target_center = target.translation().xy();

    let mut nearby_boid_count = 0_f32;
    let mut center_of_mass = target_center;
    let mut boid_positions: Vec<Vec2> = vec![];

    for (transform, movement) in &query {
        let center = transform.translation.xy();
        let distance = target_center.distance(center);
        if distance > cohesion.radius {
            continue;
        }

        let forward_velocity = Vec2::from_angle(movement.target_angle) * movement.speed;
        gizmos.arrow_2d(center, center + forward_velocity, Color::LIME_GREEN);

        center_of_mass += center;
        nearby_boid_count += 1.;

        boid_positions.push(center);
    }

    if nearby_boid_count > 0. {
        center_of_mass -= target_center;
        center_of_mass /= nearby_boid_count;

        gizmos.circle_2d(center_of_mass, 1., Color::CRIMSON);
        for boid_pos in boid_positions {
            gizmos.line_2d(center_of_mass, boid_pos, Color::DARK_GREEN);
        }

        let com_vector = center_of_mass - target_center;
        let com_velocity = com_vector.normalize() * cohesion.factor;

        gizmos.arrow_2d(target_center, center_of_mass, Color::BLUE);

        cohesion.velocity = com_velocity;
    } else {
        cohesion.velocity = Vec2::ZERO;
    }
}

fn combined_rules_system(
    mut gizmos: Gizmos,
    target: Query<(
        &GlobalTransform,
        &mut SeparationRule,
        &mut AlignmentRule,
        &mut CohesionRule,
    )>,
) {
    let (target, separation, alignment, cohesion) = target.single();
    let target_center = target.translation().xy();
    let velocities = [separation.velocity, alignment.velocity, cohesion.velocity];
    let velocity: Vec2 = velocities.iter().map(|v| v.normalize()).sum();

    gizmos.arrow_2d(
        target_center,
        target_center + velocity.normalize() * 30.,
        Color::PURPLE,
    );
}
