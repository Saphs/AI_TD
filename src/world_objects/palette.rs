use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    grid::ground_hit,
    world_objects::{
        creep_goal::{self, CreepGoal},
        knight,
        spawn_point::{self, SpawnPoint},
    },
    GameState,
};

const PALETTE_KEYS: [(KeyCode, PaletteAction); 3] = [
    (KeyCode::Digit1, PaletteAction::SpawnCreep),
    (KeyCode::Digit2, PaletteAction::SetGoal),
    (KeyCode::Digit3, PaletteAction::SetSpawnPoint),
];

pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionPalette>().add_systems(
            Update,
            (select_palette_action, use_palette_on_click).run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum PaletteAction {
    #[default]
    SpawnCreep,
    SetGoal,
    SetSpawnPoint,
}

#[derive(Resource, Default)]
struct ActionPalette {
    selected: PaletteAction,
}

fn select_palette_action(keys: Res<ButtonInput<KeyCode>>, mut palette: ResMut<ActionPalette>) {
    for (key, action) in PALETTE_KEYS {
        if keys.just_pressed(key) {
            palette.selected = action;
        }
    }
}

fn use_palette_on_click(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse: Res<ButtonInput<MouseButton>>,
    palette: Res<ActionPalette>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spawn_points: Query<(Entity, &SpawnPoint)>,
    goals: Query<Entity, With<CreepGoal>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    match palette.selected {
        PaletteAction::SpawnCreep => {
            let Ok((_, point)) = spawn_points.single() else {
                return;
            };
            knight::spawn_creep(&mut commands, &asset_server, point.position);
        }
        PaletteAction::SetGoal => {
            let Some(position) = cursor_ground_position(&windows, &cameras) else {
                return;
            };
            for entity in &goals {
                commands.entity(entity).despawn();
            }
            creep_goal::spawn_at(&mut commands, &asset_server, position);
        }
        PaletteAction::SetSpawnPoint => {
            let Some(position) = cursor_ground_position(&windows, &cameras) else {
                return;
            };
            for (entity, _) in &spawn_points {
                commands.entity(entity).despawn();
            }
            spawn_point::spawn_at(&mut commands, &asset_server, position);
        }
    }
}

fn cursor_ground_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let cursor = window.cursor_position()?;
    ground_hit(camera, camera_transform, cursor)
}
