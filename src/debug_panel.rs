use std::{collections::VecDeque, time::Instant};

use bevy::{
    camera::Viewport,
    input::{keyboard::{Key, KeyboardInput}, ButtonState},
    prelude::*,
    ui::UiTargetCamera,
    window::{CursorOptions, PrimaryWindow},
};

use crate::{
    camera_control::{CameraControlScheme, FreeRoamCamera},
    grid::GridCells,
    GameState,
};

pub struct DebugPanelPlugin;

impl Plugin for DebugPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugPanelState>()
            .init_resource::<FrameStats>()
            .add_systems(OnEnter(GameState::InGame), setup_debug_panel)
            .add_systems(First, begin_frame_stats)
            .add_systems(Last, end_frame_stats)
            .add_systems(
                Update,
                (toggle_debug_panel, update_viewports, update_debug_text)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource, Default)]
pub struct DebugPanelState {
    pub visible: bool,
}

#[derive(Resource, Default)]
struct FrameStats {
    last_start: Option<Instant>,
    current_start: Option<Instant>,
    samples: VecDeque<FrameSample>,
}

#[derive(Clone, Copy)]
struct FrameSample {
    at: Instant,
    interval: f32,
    main_work: f32,
}

fn begin_frame_stats(mut stats: ResMut<FrameStats>) {
    stats.current_start = Some(Instant::now());
}

fn end_frame_stats(mut stats: ResMut<FrameStats>) {
    let now = Instant::now();
    let Some(start) = stats.current_start.take() else {
        return;
    };
    let main_work = (now - start).as_secs_f32();
    if let Some(prev) = stats.last_start {
        stats.samples.push_back(FrameSample {
            at: start,
            interval: (start - prev).as_secs_f32(),
            main_work,
        });
    }
    stats.last_start = Some(start);
    while stats
        .samples
        .front()
        .is_some_and(|s| now.duration_since(s.at).as_secs_f32() > 1.0)
    {
        stats.samples.pop_front();
    }
}

fn latency_line(stats: &FrameStats) -> String {
    let count = stats.samples.len();
    if count == 0 {
        return "Frame interval (1 s avg): n/a".to_string();
    }
    let n = count as f32;
    let interval = stats.samples.iter().map(|s| s.interval).sum::<f32>() / n;
    let main_work = stats.samples.iter().map(|s| s.main_work).sum::<f32>() / n;
    let wait = (interval - main_work).max(0.0);
    format!(
        "Frame interval (1 s avg): {:.2} ms ({:.1} FPS)\nMain world work (1 s avg): {:.2} ms\nRender, present and wait (1 s avg): {:.2} ms",
        interval * 1000.0,
        if interval > 0.0 { 1.0 / interval } else { 0.0 },
        main_work * 1000.0,
        wait * 1000.0,
    )
}

#[derive(Component)]
struct DebugCamera;

#[derive(Component)]
struct DebugPanelRoot;

#[derive(Component)]
struct DebugText;

fn setup_debug_panel(mut commands: Commands) {
    // Second camera that renders the debug panel into the right fifth of the window
    let debug_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.05, 0.05, 0.08)),
                ..default()
            },
            DebugCamera,
            DespawnOnExit(GameState::InGame),
        ))
        .id();

    // Panel root bound to the debug camera, hidden until toggled
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Visibility::Hidden,
        UiTargetCamera(debug_camera),
        DebugPanelRoot,
        DespawnOnExit(GameState::InGame),
        children![(
            DebugText,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    ));
}

fn toggle_debug_panel(
    mut messages: MessageReader<KeyboardInput>,
    mut state: ResMut<DebugPanelState>,
) {
    for event in messages.read() {
        if event.state != ButtonState::Pressed || event.repeat {
            continue;
        }
        // "^" arrives as a dead key on layouts like German, or as a plain character otherwise
        let is_caret = matches!(&event.logical_key, Key::Character(c) if c.as_str() == "^")
            || matches!(event.logical_key, Key::Dead(Some('^')));
        if is_caret {
            state.visible = !state.visible;
        }
    }
}

fn update_viewports(
    state: Res<DebugPanelState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut main_camera: Single<&mut Camera, (With<Camera3d>, Without<DebugCamera>)>,
    mut debug_camera: Single<&mut Camera, With<DebugCamera>>,
    mut panel: Single<&mut Visibility, With<DebugPanelRoot>>,
) {
    let size = window.physical_size();

    if state.visible {
        let debug_width = size.x / 5;
        let main_width = size.x - debug_width;

        main_camera.viewport = Some(Viewport {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(main_width, size.y),
            ..default()
        });
        debug_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(main_width, 0),
            physical_size: UVec2::new(debug_width, size.y),
            ..default()
        });
        debug_camera.is_active = true;
        **panel = Visibility::Visible;
    } else {
        main_camera.viewport = None;
        debug_camera.is_active = false;
        **panel = Visibility::Hidden;
    }
}

fn update_debug_text(
    state: Res<DebugPanelState>,
    game_state: Res<State<GameState>>,
    scheme: Res<CameraControlScheme>,
    time: Res<Time>,
    cursor: Single<&CursorOptions>,
    camera: Single<(&Transform, &FreeRoamCamera), With<Camera3d>>,
    cells: Option<Res<GridCells>>,
    stats: Res<FrameStats>,
    mut text: Single<&mut Text, With<DebugText>>,
) {
    if !state.visible {
        return;
    }

    let (transform, free_roam) = *camera;
    let delta = time.delta_secs();
    let fps = if delta > 0.0 { 1.0 / delta } else { 0.0 };
    let position = transform.translation;
    let forward = transform.forward();

    let cell_line = match cells
        .as_ref()
        .and_then(|cells| cells.hovered().map(|cell| (cell, cells.state(cell))))
    {
        Some((cell, state)) => format!(
            "Hovered cell: x {}  y {}\nCell state: {:?}",
            cell.x,
            cell.y,
            state.unwrap_or_default(),
        ),
        None => "Hovered cell: none".to_string(),
    };

    text.0 = format!(
        "DEBUG PANEL\n\n\
         Game state: {:?}\n\
         Control scheme: {}\n\
         FPS: {:.0}\n\
         {}\n\n\
         Camera position: x {:.2}  y {:.2}  z {:.2}\n\
         View direction: x {:.2}  y {:.2}  z {:.2}\n\
         Yaw: {:.1} deg\n\
         Pitch: {:.1} deg\n\n\
         Cursor grab: {:?}\n\
         Cursor visible: {}\n\n\
         {}\n\n\
         Press ^ to hide this panel",
        game_state.get(),
        scheme.name(),
        fps,
        latency_line(&stats),
        position.x,
        position.y,
        position.z,
        forward.x,
        forward.y,
        forward.z,
        free_roam.yaw.to_degrees(),
        free_roam.pitch.to_degrees(),
        cursor.grab_mode,
        cursor.visible,
        cell_line,
    );
}
