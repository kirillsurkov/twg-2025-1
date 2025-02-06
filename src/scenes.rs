use bevy::prelude::*;
use game::GamePlugin;

pub mod game;
pub mod main_menu;

pub struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugin);
    }
}
