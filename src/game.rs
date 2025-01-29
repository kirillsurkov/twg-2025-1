use bevy::{prelude::*, utils::hashbrown::HashSet};
use camera::GameCameraPlugin;
use game_cursor::GameCursorPlugin;
use light_consts::lux::AMBIENT_DAYLIGHT;
use map_state::MapStatePlugin;
use primary_block::{PrimaryBlock, PrimaryBlockPlugin};
use room::{Room, RoomPlugin, RoomState};

mod camera;
mod game_cursor;
mod map_state;
mod primary_block;
mod room;
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapStatePlugin)
            .add_plugins(GameCameraPlugin(Vec3::new(0.0, 0.0, 15.0)))
            .add_plugins(PrimaryBlockPlugin)
            .add_plugins(RoomPlugin)
            .add_plugins(GameCursorPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update_state)
            .insert_resource(PlayerState::default());
    }
}

fn setup(mut commands: Commands) {
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
    commands.spawn(PrimaryBlock { x: 0, y: 0 });
}

#[derive(Resource, PartialEq, Clone, Default)]
enum PlayerState {
    #[default]
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
