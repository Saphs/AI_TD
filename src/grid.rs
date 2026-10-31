use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    transform::TransformSystems,
    window::{CursorOptions, PrimaryWindow},
};

use crate::GameState;

pub const GROUND_SIZE: f32 = 320.0;
pub const CELL_SIZE: f32 = 0.5;
const HIGHLIGHT_LIFT: f32 = 0.01;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (update_hovered_cell, render_grid_highlight)
                .chain()
                .before(TransformSystems::Propagate)
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Grid {
    pub cell_size: f32,
    pub columns: i32,
    pub rows: i32,
}

impl Grid {
    pub fn new(area: f32, cell_size: f32) -> Self {
        let count = (area / cell_size).round() as i32;
        Self {
            cell_size,
            columns: count,
            rows: count,
        }
    }

    fn half_size(&self) -> Vec2 {
        Vec2::new(self.columns as f32, self.rows as f32) * self.cell_size / 2.0
    }

    pub fn cell_at(&self, position: Vec3) -> Option<IVec2> {
        let local = (Vec2::new(position.x, position.z) + self.half_size()) / self.cell_size;
        let cell = local.floor().as_ivec2();
        if cell.x < 0 || cell.y < 0 || cell.x >= self.columns || cell.y >= self.rows {
            return None;
        }
        Some(cell)
    }

    pub fn cell_center(&self, cell: IVec2) -> Vec3 {
        let offset = (cell.as_vec2() + Vec2::splat(0.5)) * self.cell_size;
        let xz = offset - self.half_size();
        Vec3::new(xz.x, 0.0, xz.y)
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellState {
    #[default]
    Free,
    Blocked,
}

#[derive(Resource)]
pub struct GridCells {
    columns: i32,
    rows: i32,
    states: Vec<CellState>,
    hovered: Option<IVec2>,
}

impl GridCells {
    pub fn new(columns: i32, rows: i32) -> Self {
        Self {
            columns,
            rows,
            states: vec![CellState::Free; (columns * rows) as usize],
            hovered: None,
        }
    }

    fn index(&self, cell: IVec2) -> Option<usize> {
        if cell.x < 0 || cell.y < 0 || cell.x >= self.columns || cell.y >= self.rows {
            return None;
        }
        Some((cell.y * self.columns + cell.x) as usize)
    }

    pub fn state(&self, cell: IVec2) -> Option<CellState> {
        self.index(cell).map(|i| self.states[i])
    }

    #[allow(dead_code)]
    pub fn set_state(&mut self, cell: IVec2, state: CellState) {
        if let Some(i) = self.index(cell) {
            self.states[i] = state;
        }
    }

    pub fn hovered(&self) -> Option<IVec2> {
        self.hovered
    }
}

#[derive(Component)]
struct GridHighlight;

pub fn spawn_grid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(GROUND_SIZE, GROUND_SIZE));
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(images.add(grid_image())),
        uv_transform: Affine2::from_scale(Vec2::splat(GROUND_SIZE / CELL_SIZE)),
        ..default()
    });
    let highlight_mesh = meshes.add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE));
    let highlight_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.3, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let grid = Grid::new(GROUND_SIZE, CELL_SIZE);
    commands.insert_resource(GridCells::new(grid.columns, grid.rows));

    commands.spawn((
        Name::new("Grid"),
        grid,
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(GameState::InGame),
        children![
            (
                Name::new("Ground"),
                Mesh3d(ground_mesh),
                MeshMaterial3d(ground_material),
            ),
            (
                GridHighlight,
                Mesh3d(highlight_mesh),
                MeshMaterial3d(highlight_material),
                Transform::default(),
                Visibility::Hidden,
            ),
        ],
    ));
}

fn grid_image() -> Image {
    const SIZE: u32 = 64;
    const LINE: u32 = 2;
    const GROUND_SHADE: u8 = 64;
    const LINE_SHADE: u8 = 191;

    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let shade = if x < LINE || y < LINE {
                LINE_SHADE
            } else {
                GROUND_SHADE
            };
            data.extend_from_slice(&[shade, shade, shade, 255]);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn update_hovered_cell(
    windows: Query<(&Window, &CursorOptions), With<PrimaryWindow>>,
    cameras: Query<(&Camera, &Transform), With<Camera3d>>,
    grids: Query<&Grid>,
    mut cells: ResMut<GridCells>,
) {
    let Ok(grid) = grids.single() else {
        cells.hovered = None;
        return;
    };
    cells.hovered = hovered_cell(grid, &windows, &cameras);
}

fn render_grid_highlight(
    cells: Res<GridCells>,
    grids: Query<&Grid>,
    mut highlights: Query<(&mut Transform, &mut Visibility), With<GridHighlight>>,
) {
    let Ok(grid) = grids.single() else {
        return;
    };
    let Ok((mut transform, mut visibility)) = highlights.single_mut() else {
        return;
    };

    match cells
        .hovered()
        .filter(|cell| cells.state(*cell) == Some(CellState::Free))
    {
        Some(cell) => {
            transform.translation = grid.cell_center(cell) + Vec3::Y * HIGHLIGHT_LIFT;
            *visibility = Visibility::Visible;
        }
        None => *visibility = Visibility::Hidden,
    }
}

fn hovered_cell(
    grid: &Grid,
    windows: &Query<(&Window, &CursorOptions), With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &Transform), With<Camera3d>>,
) -> Option<IVec2> {
    let (window, cursor_options) = windows.single().ok()?;
    if !cursor_options.visible {
        return None;
    }
    let (camera, transform) = cameras.single().ok()?;
    let cursor = window.cursor_position()?;
    let point = ground_hit(camera, &GlobalTransform::from(*transform), cursor)?;
    grid.cell_at(point)
}

pub fn ground_hit(camera: &Camera, camera_transform: &GlobalTransform, cursor: Vec2) -> Option<Vec3> {
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    if ray.direction.y.abs() < f32::EPSILON {
        return None;
    }

    let distance = -ray.origin.y / ray.direction.y;
    if distance < 0.0 {
        return None;
    }

    Some(ray.origin + *ray.direction * distance)
}
