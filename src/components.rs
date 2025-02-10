use background::BackgroundPlugin;
use bevy::prelude::*;
use collisions::CollisionsPlugin;
use game_button::GameButtonPlugin;
use material_modifier::MaterialModifierPlugin;

pub mod background;
pub mod mouse_event;
pub mod collisions;
pub mod game_button;
pub mod material_modifier;
pub mod procedural_material;

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BackgroundPlugin)
            .add_plugins(CollisionsPlugin)
            .add_plugins(GameButtonPlugin)
            .add_plugins(MaterialModifierPlugin::<StandardMaterial, StandardMaterial>::default());
    }
}
