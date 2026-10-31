use bevy::prelude::*;

use crate::{model_loader::ModelLoader, GameState};

const DEFAULT_POSITION: Vec3 = Vec3::new(50.0, 0.0, 50.0);
const MODEL_PATH: &str = "pixellabs-wooden-signpost-3d-2944.glb";
// The signpost mesh is centred on its origin, so lift it by half its 1.12 m height
const MODEL_LIFT: f32 = 0.561;

pub struct SpawnPointPlugin;

impl Plugin for SpawnPointPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_default);
    }
}

#[derive(Component)]
pub struct SpawnPoint {
    pub position: Vec3,
}

fn spawn_default(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_at(&mut commands, &asset_server, DEFAULT_POSITION);
}

pub fn spawn_at(commands: &mut Commands, asset_server: &AssetServer, position: Vec3) {
    let entity = ModelLoader {
        path: MODEL_PATH,
        transform: Transform::from_translation(position + Vec3::Y * MODEL_LIFT),
    }
    .spawn(commands, asset_server);

    commands.entity(entity).insert(SpawnPoint { position });
}
