use bevy::prelude::*;
use cargo_count::GameUiCargoCountPlugin;
use container::GameUiContainerPlugin;
use container_item::GameUiContainerItemPlugin;
use header::GameUiHeaderPlugin;
use power_bar::GameUiPowerBarPlugin;

pub mod cargo_count;
pub mod container;
pub mod container_item;
pub mod header;
pub mod palette;
pub mod power_bar;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameUiHeaderPlugin)
            .add_plugins(GameUiContainerPlugin)
            .add_plugins(GameUiContainerItemPlugin)
            .add_plugins(GameUiCargoCountPlugin)
            .add_plugins(GameUiPowerBarPlugin);
    }
}
