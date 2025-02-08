use std::f32::consts::PI;

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use components::ComponentsPlugin;
use noisy_bevy::NoisyShaderPlugin;
use rand::Rng;
use scenes::AppScenesPlugin;

mod components;
mod scenes;

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

        Quat::from_xyzw(
            s * angle1.sin(),
            s * angle1.cos(),
            t * angle2.sin(),
            t * angle2.cos(),
        )
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Avaruus".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(ComponentsPlugin)
        .add_plugins(AppScenesPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(AmbientLight::NONE)
        .insert_resource(FpsOverlayConfig {
            // enabled: false,
            ..Default::default()
        })
        .run();
}
