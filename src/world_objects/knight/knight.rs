#[path = "behavior.rs"]
pub mod behavior;

use bevy::prelude::*;

use crate::{
    model_loader::ModelLoader,
    world_objects::creep_goal,
    GameState,
};

use behavior::MoveToGoal;

// Reference height of a normal NPC, 1 unit = 1 m
pub const NPC_HEIGHT: f32 = 1.8;
const KNIGHT_PATH: &str = "nickpanek-knight-1643.glb";
const KNIGHT_NATIVE_HEIGHT: f32 = 0.254;
const KNIGHT_SCALE: f32 = NPC_HEIGHT / KNIGHT_NATIVE_HEIGHT;
const CREEP_SPEED: f32 = 3.0;

pub struct KnightPlugin;

impl Plugin for KnightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (behavior::update_creep_targets, behavior::move_to_goal)
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Creep;

pub fn spawn_reference_knight(commands: &mut Commands, asset_server: &AssetServer) {
    ModelLoader {
        path: KNIGHT_PATH,
        transform: Transform::from_xyz(-3.5, 0.0, 0.0).with_scale(Vec3::splat(KNIGHT_SCALE)),
    }
    .spawn(commands, asset_server);
}

pub fn spawn_creep(commands: &mut Commands, asset_server: &AssetServer, position: Vec3) {
    let entity = ModelLoader {
        path: KNIGHT_PATH,
        transform: Transform::from_translation(position).with_scale(Vec3::splat(KNIGHT_SCALE)),
    }
    .spawn(commands, asset_server);

    commands.entity(entity).insert((
        Creep,
        MoveToGoal {
            goal: creep_goal::DEFAULT_POSITION,
            speed: CREEP_SPEED,
        },
    ));
}
