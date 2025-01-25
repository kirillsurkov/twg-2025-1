use background::{BackgroundPlugin, RenderBackground};
use bevy::{
    core_pipeline::{
        bloom::Bloom, oit::OrderIndependentTransparencySettings, tonemapping::DebandDither,
    },
    dev_tools::fps_overlay::FpsOverlayPlugin,
    input::mouse::MouseWheel,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use light_consts::lux::AMBIENT_DAYLIGHT;
use mipmaps::MipmapGeneratorPlugin;
use modify_material::ModifyMaterialPlugin;
use noisy_bevy::NoisyShaderPlugin;
use room::{Room, RoomPlugin};
use update_material_textures::UpdateMaterialTexturesPlugin;

mod background;
mod mipmaps;
mod modify_material;
mod procedural_material;
mod room;
mod update_material_textures;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(UpdateMaterialTexturesPlugin::<StandardMaterial>::default())
        .add_plugins(ModifyMaterialPlugin::<StandardMaterial>::default())
        .add_plugins(MipmapGeneratorPlugin)
        .add_plugins(BackgroundPlugin::new("stars.wgsl"))
        .add_plugins(RoomPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ray_cast_visibility: RayCastVisibility::Any,
        })
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
            clear_color: ClearColorConfig::None,
            ..Default::default()
        },
        RenderBackground,
        RayCastPickable,
        Msaa::Off,
        OrderIndependentTransparencySettings {
            layer_count: 32,
            alpha_threshold: 0.01,
        },
        Bloom::NATURAL,
        DebandDither::Enabled,
        Transform::from_xyz(0.0, 0.0, 15.0),
        TargetPos(Vec3::new(0.0, 0.0, 15.0)),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: AMBIENT_DAYLIGHT * 0.1,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(1.0, 1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: AMBIENT_DAYLIGHT * 0.1,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(-1.0, -1.0, 10.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn(Room { x: -1, y: 0 });
    commands.spawn(Room { x: 0, y: 0 });
    commands.spawn(Room { x: 1, y: 0 });
    commands.spawn(Room { x: 0, y: 1 });
}

fn camera_control(
    mut wheel: EventReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut TargetPos), With<Camera3d>>,
    time: Res<Time>,
) {
    let (mut camera, mut target_pos) = camera.single_mut();

    for event in wheel.read() {
        **target_pos = **target_pos - Vec3::new(0.0, 0.0, event.y);
    }

    target_pos.z = target_pos.z.max(2.0);

    let diff = **target_pos - camera.translation;
    camera.translation += diff * time.delta_secs() / 0.1;
}
