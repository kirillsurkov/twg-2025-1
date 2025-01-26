use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::GamePlugin;
use mipmaps::MipmapGeneratorPlugin;
use modify_material::ModifyMaterialPlugin;
use noisy_bevy::NoisyShaderPlugin;
use update_material_textures::UpdateMaterialTexturesPlugin;

mod background;
mod game;
mod mipmaps;
mod modify_material;
mod procedural_material;
mod room;
mod update_material_textures;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(UpdateMaterialTexturesPlugin::<StandardMaterial>::default())
        .add_plugins(ModifyMaterialPlugin::<StandardMaterial>::default())
        // .add_plugins(MipmapGeneratorPlugin)
        .add_plugins(GamePlugin)
        .insert_resource(AmbientLight::NONE)
        .run();
}
