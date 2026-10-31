use bevy::prelude::*;

use super::Creep;
use crate::world_objects::creep_goal::CreepGoal;

// glTF models usually face +Z, while Bevy's look_to aims -Z at the target
const MODEL_FORWARD_OFFSET: f32 = std::f32::consts::PI;

#[derive(Component)]
pub struct MoveToGoal {
    pub goal: Vec3,
    pub speed: f32,
}

pub fn update_creep_targets(
    goals: Query<&CreepGoal>,
    mut creeps: Query<&mut MoveToGoal, With<Creep>>,
) {
    let Ok(goal) = goals.single() else {
        return;
    };

    for mut behavior in &mut creeps {
        if behavior.goal != goal.position {
            behavior.goal = goal.position;
        }
    }
}

pub fn move_to_goal(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &MoveToGoal), With<Creep>>,
) {
    for (entity, mut transform, behavior) in &mut query {
        let to_goal = behavior.goal - transform.translation;
        let step = behavior.speed * time.delta_secs();
        if to_goal.length() <= step {
            commands.entity(entity).despawn();
        } else {
            let direction = to_goal.normalize();
            transform.translation += direction * step;
            transform.look_to(direction, Vec3::Y);
            transform.rotate_local_y(MODEL_FORWARD_OFFSET);
        }
    }
}
