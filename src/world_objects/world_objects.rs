#[path = "creep_goal/creep_goal.rs"]
pub mod creep_goal;
#[path = "knight/knight.rs"]
pub mod knight;
#[path = "palette.rs"]
pub mod palette;
#[path = "spawn_point/spawn_point.rs"]
pub mod spawn_point;

use bevy::prelude::*;

pub struct WorldObjectsPlugin;

impl Plugin for WorldObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            knight::KnightPlugin,
            spawn_point::SpawnPointPlugin,
            creep_goal::CreepGoalPlugin,
            palette::PalettePlugin,
        ));
    }
}
