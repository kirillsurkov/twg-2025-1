use bevy::{
    core_pipeline::{
        bloom::Bloom, oit::OrderIndependentTransparencySettings, tonemapping::DebandDither,
    },
    input::mouse::MouseWheel,
    prelude::*,
};

use crate::background::{BackgroundPlugin, RenderBackground};

pub struct GameCameraPlugin(pub Vec3);

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BackgroundPlugin::new("stars.wgsl"))
            .add_systems(Startup, setup)
            .add_systems(Update, camera_control)
            .insert_resource(TargetPos(self.0));
    }
}

#[derive(Resource, Deref, DerefMut)]
struct TargetPos(Vec3);

fn setup(mut commands: Commands, target_pos: Res<TargetPos>) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::None,
            ..Default::default()
        },
        RenderBackground,
        Msaa::Off,
        OrderIndependentTransparencySettings {
            layer_count: 32,
            alpha_threshold: 0.01,
        },
        Bloom::NATURAL,
        DebandDither::Enabled,
        Transform::from_translation(target_pos.0),
    ));
}

fn camera_control(
    mut wheel: EventReader<MouseWheel>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
    mut target_pos: ResMut<TargetPos>,
    time: Res<Time>,
) {
    let mut camera = camera.single_mut();

    for event in wheel.read() {
        **target_pos = **target_pos - Vec3::new(0.0, 0.0, event.y);
    }

    target_pos.z = target_pos.z.max(2.0).min(40.0);

    let diff = **target_pos - camera.translation;
    camera.translation += diff * time.delta_secs() / 0.1;
}
