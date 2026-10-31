use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{observe, Activate, Button},
};

use crate::GameState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                update_button_hover.run_if(in_state(GameState::MainMenu)),
            );
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

fn update_button_hover(
    mut buttons: Query<(&Hovered, &mut BackgroundColor), (With<Button>, Changed<Hovered>)>,
) {
    for (hovered, mut color) in &mut buttons {
        *color = if hovered.get() {
            HOVERED_BUTTON.into()
        } else {
            NORMAL_BUTTON.into()
        };
    }
}

fn setup_main_menu(mut commands: Commands) {
    // 2D Camera scoped to MainMenu (despawns automatically on state exit)
    commands.spawn((
        Camera2d,
        DespawnOnExit(GameState::MainMenu),
    ));

    // Full-screen container that centers the title and buttons
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        DespawnOnExit(GameState::MainMenu),
        children![
            (
                Text::new("MAIN MENU"),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.2)),
            ),
            (
                Button,
                Hovered::default(),
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                observe(|_: On<Activate>, mut next_state: ResMut<NextState<GameState>>| {
                    next_state.set(GameState::InGame);
                }),
                children![(
                    Text::new("Play"),
                    TextFont {
                        font_size: FontSize::Px(32.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
            (
                Button,
                Hovered::default(),
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                observe(|_: On<Activate>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                }),
                children![(
                    Text::new("Quit"),
                    TextFont {
                        font_size: FontSize::Px(32.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}
