use std::f32::consts::PI;

use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::GamePlugin;
use material_modifier::MaterialModifierPlugin;
use noisy_bevy::NoisyShaderPlugin;
use rand::Rng;
use rand_distr::{Distribution, Normal, StandardNormal};
use update_material_textures::UpdateMaterialTexturesPlugin;

mod background;
mod game;
mod material_modifier;
mod procedural_material;
mod update_material_textures;

trait RandomRotation {
    fn random() -> Quat;
}

impl RandomRotation for Quat {
    fn random() -> Quat {
        let mut rng = rand::rng();
        let u: f32 = rng.random();
        let v: f32 = rng.random();
        let w: f32 = rng.random();

        let s = (1.0 - u).sqrt();
        let t = u.sqrt();
        let angle1 = 2.0 * PI * v;
        let angle2 = 2.0 * PI * w;

        let x = s * angle1.sin();
        let y = s * angle1.cos();
        let z = t * angle2.sin();
        let w_component = t * angle2.cos();

        Quat::from_xyzw(x, y, z, w_component)
    }
}

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
