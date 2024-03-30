use crate::plugins::components::{AlignmentRule, BoidMovement, CohesionRule, SeparationRule};
use bevy::prelude::*;

use super::startup::BOID_COUNT;

pub struct RulesPlugin;

impl Plugin for RulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                separation_system,
                alignment_system,
                cohesion_system,
                velocity_system,
            )
                .chain(),
        );
    }
}

fn separation_system(mut query: Query<(&GlobalTransform, &mut SeparationRule)>) {
    let mut velocities: [Option<Vec2>; BOID_COUNT] = [Option::None; BOID_COUNT];
    for (current_transform, current_separation) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut velocity = Vec2::ZERO;
        // max magnitude
        const MAGNITUDE: f32 = 60.;

        for (transform, separation) in &query {
            if separation.id == current_separation.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_separation.radius {
                continue;
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

            velocities[current_separation.id] = Some(velocity);
        }
    }

    for (_, mut separation) in &mut query {
        if let Some(vel) = velocities[separation.id] {
            separation.velocity = vel;
        }
    }
}

fn alignment_system(mut query: Query<(&Transform, &mut AlignmentRule)>) {
    let mut velocities: [Option<Vec2>; BOID_COUNT] = [Option::None; BOID_COUNT];
    for (current_transform, current_alignment) in &query {
        let current_center = current_transform.translation.xy();
        let mut nearby_boid_count = 0_f32;
        let mut velocity = Vec2::ZERO;

        for (transform, alignment) in &query {
            if alignment.id == current_alignment.id {
                continue;
            }

            let center = transform.translation.xy();
            let distance = current_center.distance(center);
            if distance > current_alignment.radius {
                continue;
            }

            let boid_velocity = (transform.rotation * Vec3::Y).xy();

            velocity += boid_velocity;
            nearby_boid_count += 1.;
        }

        if nearby_boid_count > 0. {
            velocity /= nearby_boid_count;
            velocity *= current_alignment.factor;

            velocities[current_alignment.id] = Some(velocity);
        }
    }

    for (_, mut alignment) in &mut query {
        if let Some(vel) = velocities[alignment.id] {
            alignment.velocity = vel;
        }
    }
}

fn cohesion_system(mut query: Query<(&GlobalTransform, &mut CohesionRule)>) {
    let mut velocities: [Option<Vec2>; BOID_COUNT] = [Option::None; BOID_COUNT];
    for (current_transform, current_cohesion) in &query {
        let current_center = current_transform.translation().xy();
        let mut nearby_boid_count = 0_f32;
        let mut center_of_mass = current_center;
        let mut boid_positions: Vec<Vec2> = vec![];

        for (transform, cohesion) in &query {
            if cohesion.id == current_cohesion.id {
                continue;
            }

            let center = transform.translation().xy();
            let distance = current_center.distance(center);
            if distance > current_cohesion.radius {
                continue;
            }

            center_of_mass += center;
            nearby_boid_count += 1.;

            boid_positions.push(center);
        }

        if nearby_boid_count > 0. {
            center_of_mass -= current_center;
            center_of_mass /= nearby_boid_count;

            let com_vector = center_of_mass - current_center;
            let com_velocity = com_vector.normalize() * current_cohesion.factor;

            velocities[current_cohesion.id] = Some(com_velocity);
        }
    }

    for (_, mut cohesion) in &mut query {
        if let Some(vel) = velocities[cohesion.id] {
            cohesion.velocity = vel;
        }
    }
}

fn velocity_system(
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
