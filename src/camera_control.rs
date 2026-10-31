use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollPixelsPerLine},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::GameState;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraControlScheme>()
            .add_systems(OnEnter(GameState::InGame), grab_cursor)
            .add_systems(OnExit(GameState::InGame), release_cursor)
            .add_systems(
                Update,
                (
                    toggle_cursor_grab,
                    switch_camera_scheme,
                    free_roam_camera.run_if(|scheme: Res<CameraControlScheme>| {
                        *scheme == CameraControlScheme::FreeRoam
                    }),
                    rts_classic_camera.run_if(|scheme: Res<CameraControlScheme>| {
                        *scheme == CameraControlScheme::RtsClassic
                    }),
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CameraControlScheme {
    #[default]
    FreeRoam,
    RtsClassic,
}

const SCHEME_KEYS: [(KeyCode, CameraControlScheme); 2] = [
    (KeyCode::F1, CameraControlScheme::FreeRoam),
    (KeyCode::F2, CameraControlScheme::RtsClassic),
];

impl CameraControlScheme {
    pub fn name(self) -> &'static str {
        match self {
            CameraControlScheme::FreeRoam => "Free Roam (F1)",
            CameraControlScheme::RtsClassic => "RTS Classic (F2)",
        }
    }
}

const RTS_PITCH_DEGREES: f32 = -55.0;
const RTS_EDGE_MARGIN: f32 = 20.0;
const RTS_PAN_SPEED: f32 = 12.0;
const RTS_ZOOM_MIN: f32 = 10.0;
const RTS_ZOOM_MAX: f32 = 200.0;
const RTS_ZOOM_STEP: f32 = 8.0;
const RTS_START_HEIGHT: f32 = 33.0;

fn rts_camera_rotation() -> Quat {
    Quat::from_euler(EulerRot::YXZ, 0.0, RTS_PITCH_DEGREES.to_radians(), 0.0)
}

#[derive(Component)]
pub struct FreeRoamCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub move_speed: f32,
    pub look_sensitivity: f32,
}

impl FreeRoamCamera {
    pub fn from_transform(transform: &Transform) -> Self {
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        Self {
            yaw,
            pitch,
            move_speed: 8.0,
            look_sensitivity: 0.003,
        }
    }
}

#[derive(Component)]
pub struct RtsCamera {
    pub focus: Vec3,
    pub distance: f32,
}

impl RtsCamera {
    pub fn from_transform(transform: &Transform) -> Self {
        let forward = *transform.forward();
        let ground = transform.translation + forward * (transform.translation.y / -forward.y);
        Self {
            focus: Vec3::new(ground.x, 0.0, ground.z),
            distance: RTS_START_HEIGHT / (-RTS_PITCH_DEGREES).to_radians().sin(),
        }
    }
}

fn grab_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Confined;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}

fn switch_camera_scheme(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scheme: ResMut<CameraControlScheme>,
    mut cursor: Single<&mut CursorOptions>,
) {
    let Some(next) = SCHEME_KEYS
        .iter()
        .find(|(key, _)| keyboard.just_pressed(*key))
        .map(|(_, scheme)| *scheme)
    else {
        return;
    };

    if *scheme == next {
        return;
    }

    *scheme = next;
    match next {
        CameraControlScheme::FreeRoam => {
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Confined;
        }
        CameraControlScheme::RtsClassic => {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::Confined;
        }
    }
}

fn toggle_cursor_grab(keyboard: Res<ButtonInput<KeyCode>>, mut cursor: Single<&mut CursorOptions>) {
    if keyboard.just_pressed(KeyCode::Tab) {
        let grab = matches!(cursor.grab_mode, CursorGrabMode::None);
        cursor.visible = !grab;
        cursor.grab_mode = if grab {
            CursorGrabMode::Confined
        } else {
            CursorGrabMode::None
        };
    }
}

fn free_roam_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    cursor: Single<&CursorOptions>,
    scheme: Res<CameraControlScheme>,
    mut cameras: Query<(&mut Transform, &mut FreeRoamCamera)>,
) {
    let looking = !matches!(cursor.grab_mode, CursorGrabMode::None);

    for (mut transform, mut camera) in &mut cameras {
        if scheme.is_changed() || !looking {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            camera.yaw = yaw;
            camera.pitch = pitch;
        }

        if looking {
            camera.yaw -= mouse_motion.delta.x * camera.look_sensitivity;
            camera.pitch = (camera.pitch - mouse_motion.delta.y * camera.look_sensitivity)
                .clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
        }

        let forward = *transform.forward();
        let right = *transform.right();
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += right;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        if direction != Vec3::ZERO {
            transform.translation += direction.normalize() * camera.move_speed * time.delta_secs();
        }
    }
}

fn rts_classic_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    scroll: Res<AccumulatedMouseScroll>,
    scheme: Res<CameraControlScheme>,
    mut cameras: Query<(&mut Transform, &mut RtsCamera), With<Camera3d>>,
) {
    let mut direction = Vec2::ZERO;

    if let Some(cursor) = window.cursor_position() {
        let width = window.width();
        let height = window.height();
        if cursor.x <= RTS_EDGE_MARGIN {
            direction.x -= 1.0;
        }
        if cursor.x >= width - RTS_EDGE_MARGIN {
            direction.x += 1.0;
        }
        if cursor.y <= RTS_EDGE_MARGIN {
            direction.y -= 1.0;
        }
        if cursor.y >= height - RTS_EDGE_MARGIN {
            direction.y += 1.0;
        }
    }

    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y += 1.0;
    }

    let movement = direction.normalize_or_zero() * RTS_PAN_SPEED * time.delta_secs();

    let scroll_lines = scroll.to_lines(&MouseScrollPixelsPerLine::default()).delta.y;

    for (mut transform, mut rts) in &mut cameras {
        if scheme.is_changed() {
            *rts = RtsCamera::from_transform(&transform);
        }

        rts.distance =
            (rts.distance - scroll_lines * RTS_ZOOM_STEP).clamp(RTS_ZOOM_MIN, RTS_ZOOM_MAX);
        rts.focus += Vec3::new(movement.x, 0.0, movement.y);

        let rotation = rts_camera_rotation();
        transform.rotation = rotation;
        transform.translation = rts.focus + rotation * Vec3::Z * rts.distance;
    }
}
