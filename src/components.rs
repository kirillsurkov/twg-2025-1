use background::BackgroundPlugin;
use bevy::prelude::*;
use material_modifier::MaterialModifierPlugin;
use update_material_textures::UpdateMaterialTexturesPlugin;

pub mod background;
pub mod material_modifier;
pub mod mipmaps;
pub mod procedural_material;
pub mod update_material_textures;

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BackgroundPlugin)
            .add_plugins(MaterialModifierPlugin::<StandardMaterial, StandardMaterial>::default())
            .add_plugins(UpdateMaterialTexturesPlugin::<StandardMaterial>::default());
    }
}
