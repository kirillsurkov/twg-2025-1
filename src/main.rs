use std::f32::consts::PI;

use bevy::{
    audio::Volume,
    core_pipeline::{bloom::Bloom, oit::OrderIndependentTransparencySettings},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    window::{WindowMode, WindowResized},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use components::ComponentsPlugin;
use noisy_bevy::NoisyShaderPlugin;
use rand::Rng;
use scenes::{main_menu::settings::GameSettings, AppScenesPlugin};

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

fn on_resize(mut commands: Commands, mut resize_reader: EventReader<WindowResized>) {
    for e in resize_reader.read() {
        commands.insert_resource(UiScale((e.width / 1920.0).min(1.0)));
    }
}

fn apply_settings(
    settings: Res<GameSettings>,
    mut playbacks: Query<&mut PlaybackSettings>,
    mut oits: Query<&mut OrderIndependentTransparencySettings>,
    mut blooms: Query<&mut Bloom>,
    mut window: Query<&mut Window>,
) {
    for mut playback in playbacks.iter_mut() {
        playback.volume = Volume::new(settings.music_volume);
    }

    for mut oit in oits.iter_mut() {
        if oit.layer_count != settings.oit_layers as i32 {
            oit.layer_count = settings.oit_layers as i32;
        }
    }

    for mut bloom in blooms.iter_mut() {
        bloom.intensity = settings.bloom as u32 as f32 * 0.15;
    }

    if let Ok(mut window) = window.get_single_mut() {
        let is_fullscreen = matches!(window.mode, WindowMode::Fullscreen(_));
        if is_fullscreen && !settings.fullscreen {
            window.mode = WindowMode::Windowed;
        }
        if !is_fullscreen && settings.fullscreen {
            window.mode = WindowMode::Fullscreen(MonitorSelection::Current);
        }
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
        // .add_plugins(WorldInspectorPlugin::new())
        // .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(ComponentsPlugin)
        .add_plugins(AppScenesPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(AmbientLight::NONE)
        // .insert_resource(FpsOverlayConfig {
        //     // enabled: false,
        //     ..Default::default()
        // })
        .add_systems(PreUpdate, (on_resize, apply_settings))
        .run();
}
