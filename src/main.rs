mod camera_control;
mod debug_panel;
mod game_over;
mod grid;
mod in_game;
mod main_menu;
mod model_loader;
#[path = "world_objects/world_objects.rs"]
mod world_objects;

use bevy::{
    prelude::*,
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    window::WindowMode,
};

// Define the 3 distinct game states (scenes)
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
}

fn perf_logs(app: &mut App) {
    app.add_plugins((
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        SystemInformationDiagnosticsPlugin,
    ));
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::Fullscreen(
                            MonitorSelection::Primary,
                            VideoModeSelection::Current,
                        ),
                        ..default()
                    }),
                    ..default()
                })
                // Render the frame that was just simulated instead of the previous one,
                // so the hover highlight is drawn from the same cursor sample as the frame.
                .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
        )
        // .add_plugins(perf_logs)
        // Initialize the state
        .init_state::<GameState>()
        .add_systems(Update, exit_on_escape)
        .add_plugins((
            camera_control::CameraControlPlugin,
            world_objects::WorldObjectsPlugin,
            debug_panel::DebugPanelPlugin,
            grid::GridPlugin,
            main_menu::MainMenuPlugin,
            in_game::InGamePlugin,
            game_over::GameOverPlugin,
        ))
        .run();
}

fn exit_on_escape(keyboard: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
