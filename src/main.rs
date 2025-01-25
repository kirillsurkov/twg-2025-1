use background::BackgroundPlugin;
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::DebandDither},
    dev_tools::fps_overlay::FpsOverlayPlugin,
    input::mouse::MouseWheel,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use light_consts::lux::AMBIENT_DAYLIGHT;
use mipmaps::MipmapGeneratorPlugin;
use noisy_bevy::NoisyShaderPlugin;
use room::{Room, RoomPlugin};
use update_material_textures::UpdateMaterialTexturesPlugin;

mod background;
mod mipmaps;
mod procedural_material;
mod room;
mod update_material_textures;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(UpdateMaterialTexturesPlugin::<StandardMaterial>::default())
        .add_plugins(MipmapGeneratorPlugin)
        .add_plugins(BackgroundPlugin)
        .add_plugins(RoomPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .insert_resource(AmbientLight::NONE)
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct TargetPos(Vec3);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Bloom::NATURAL,
        DebandDither::Enabled,
        Transform::from_xyz(0.0, 15.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-90.0_f32.to_radians())),
        TargetPos(Vec3::new(0.0, 15.0, 0.0)),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: AMBIENT_DAYLIGHT * 0.1,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(1.0, 10.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: AMBIENT_DAYLIGHT * 0.1,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-1.0, 10.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((Room, Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0))));
    commands.spawn((Room, Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))));
    commands.spawn((Room, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0))));
}

fn camera_control(
    mut wheel: EventReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut TargetPos), With<Camera3d>>,
    time: Res<Time>,
) {
    let (mut camera, mut target_pos) = camera.single_mut();

    for event in wheel.read() {
        **target_pos = **target_pos - Vec3::new(0.0, event.y, 0.0);
    }

    target_pos.y = target_pos.y.max(2.0);

    let diff = **target_pos - camera.translation;
    camera.translation += diff * time.delta_secs() / 0.1;
}
