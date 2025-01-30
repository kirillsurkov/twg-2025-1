use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::GamePlugin;
use material_modifier::MaterialModifierPlugin;
use noisy_bevy::NoisyShaderPlugin;
use update_material_textures::UpdateMaterialTexturesPlugin;

mod background;
mod game;
mod material_modifier;
mod procedural_material;
mod update_material_textures;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(UpdateMaterialTexturesPlugin::<StandardMaterial>::default())
        .add_plugins(MaterialModifierPlugin::<StandardMaterial, StandardMaterial>::default())
        .add_plugins(GamePlugin)
        .insert_resource(AmbientLight::NONE)
        .run();
}
