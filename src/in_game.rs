use bevy::{prelude::*, ui::IsDefaultUiCamera};

use crate::{
    camera_control::{FreeRoamCamera, RtsCamera},
    grid,
    world_objects::knight::{self, NPC_HEIGHT},
    GameState,
};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_in_game)
            .add_systems(Update, update_in_game.run_if(in_state(GameState::InGame)));
    }
}

fn setup_in_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // 3D Camera for TD gameplay
    let camera_transform = Transform::from_xyz(0.0, 12.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3d::default(),
        FreeRoamCamera::from_transform(&camera_transform),
        RtsCamera::from_transform(&camera_transform),
        camera_transform,
        IsDefaultUiCamera,
        DespawnOnExit(GameState::InGame),
    ));

    // Directional light (Sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        DespawnOnExit(GameState::InGame),
    ));

    // Grid entity with the dark gray ground plane and 0.5 m cells
    grid::spawn_grid(&mut commands, &mut meshes, &mut materials, &mut images);

    // Sample Tower Entity
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 3.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.8))),
        Transform::from_xyz(2.0, 1.5, 0.0),
        DespawnOnExit(GameState::InGame),
    ));

    // Reference NPC, 1.8 m tall (1 unit = 1 m)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, NPC_HEIGHT, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.8, 0.7))),
        Transform::from_xyz(-2.0, NPC_HEIGHT / 2.0, 0.0),
        DespawnOnExit(GameState::InGame),
    ));

    // Knight model beside the NPC box, scaled to 1.8 m
    knight::spawn_reference_knight(&mut commands, &asset_server);

    // In-game HUD
    commands.spawn((
        Text::new("IN-GAME\nPress [G] to Trigger Game Over"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        DespawnOnExit(GameState::InGame),
    ));
}

fn update_in_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        next_state.set(GameState::GameOver);
    }
}
