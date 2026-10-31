use bevy::prelude::*;

use crate::{model_loader::ModelLoader, GameState};

pub const DEFAULT_POSITION: Vec3 = Vec3::ZERO;
const MODEL_PATH: &str = "gustavorezende-celtic-3412.glb";

pub struct CreepGoalPlugin;

impl Plugin for CreepGoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_default);
    }
}

#[derive(Component)]
pub struct CreepGoal {
    pub position: Vec3,
}

fn spawn_default(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_at(&mut commands, &asset_server, DEFAULT_POSITION);
}

pub fn spawn_at(commands: &mut Commands, asset_server: &AssetServer, position: Vec3) {
    let entity = ModelLoader {
        path: MODEL_PATH,
        transform: Transform::from_translation(position),
    }
    .spawn(commands, asset_server);

    commands.entity(entity).insert(CreepGoal { position });
}
