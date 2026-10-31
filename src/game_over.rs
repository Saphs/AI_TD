use bevy::prelude::*;

use crate::GameState;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(Update, update_game_over.run_if(in_state(GameState::GameOver)));
    }
}

fn setup_game_over(mut commands: Commands) {
    // 2D Camera for Game Over screen
    commands.spawn((
        Camera2d,
        DespawnOnExit(GameState::GameOver),
    ));

    commands.spawn((
        Text::new("GAME OVER\nPress [R] to Return to Menu"),
        TextFont {
            font_size: FontSize::Px(40.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(32.0),
            top: Val::Percent(45.0),
            ..default()
        },
        DespawnOnExit(GameState::GameOver),
    ));
}

fn update_game_over(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::MainMenu);
    }
}
