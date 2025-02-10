use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use crate::components::material_modifier::MaterialModifierPlugin;

pub struct BuildMaterialPlugin;

impl Plugin for BuildMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExtendedBuildMaterial>::default())
            .add_plugins(MaterialModifierPlugin::<
                StandardMaterial,
                ExtendedBuildMaterial,
            >::default());
    }
}

#[derive(Debug, Clone, ShaderType, Reflect)]
pub struct BuildMaterialSettings {
    pub created: f32,
    pub color: LinearRgba,
    pub direction: f32,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct BuildMaterial {
    #[uniform(100)]
    pub settings: BuildMaterialSettings,
}

pub type ExtendedBuildMaterial = ExtendedMaterial<StandardMaterial, BuildMaterial>;

impl MaterialExtension for BuildMaterial {
    fn fragment_shader() -> ShaderRef {
        "build.wgsl".into()
    }
}
