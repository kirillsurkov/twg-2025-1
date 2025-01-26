use bevy::{
    core_pipeline::{
        bloom::Bloom, oit::OrderIndependentTransparencySettings, tonemapping::DebandDither,
    },
    input::mouse::MouseWheel,
    prelude::*,
};
use game_cursor::GameCursorPlugin;
use light_consts::lux::AMBIENT_DAYLIGHT;
use room::RoomPlugin;

use crate::background::{BackgroundPlugin, RenderBackground};

pub mod game_cursor;
mod room;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BackgroundPlugin::new("stars.wgsl"))
            .add_plugins(RoomPlugin)
            .add_plugins(GameCursorPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, camera_control)
            .add_systems(Update, update_state)
            .insert_resource(PlayerState::Idle);
    }
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

#[derive(Resource)]
enum PlayerState {
    Idle,
    Construct,
    Destruct,
}

fn update_state(mut player_state: ResMut<PlayerState>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        *player_state = if let PlayerState::Construct = *player_state {
            PlayerState::Idle
        } else {
            PlayerState::Construct
        }
    }

    if keyboard.just_pressed(KeyCode::KeyD) {
        *player_state = if let PlayerState::Destruct = *player_state {
            PlayerState::Idle
        } else {
            PlayerState::Destruct
        };
    }
}
