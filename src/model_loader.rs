use bevy::{gltf::GltfAssetLabel, prelude::*};

use crate::GameState;

pub struct ModelLoader {
    pub path: &'static str,
    pub transform: Transform,
}

impl ModelLoader {
    pub fn spawn(self, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let scene: Handle<WorldAsset> =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(self.path));

        commands
            .spawn((
                WorldAssetRoot(scene),
                self.transform,
                DespawnOnExit(GameState::InGame),
            ))
            .id()
    }
}
