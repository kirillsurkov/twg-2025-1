use background::BackgroundPlugin;
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::DebandDither},
    dev_tools::fps_overlay::FpsOverlayPlugin,
    prelude::*,
};
use noisy_bevy::NoisyShaderPlugin;

mod background;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(BackgroundPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Bloom::NATURAL,
        DebandDither::Enabled,
        Transform::from_xyz(0.0, 15.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
