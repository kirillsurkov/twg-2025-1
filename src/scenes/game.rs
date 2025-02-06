use bevy::{audio::PlaybackMode, dev_tools::fps_overlay::FpsOverlayConfig, prelude::*};
use camera::GameCameraPlugin;
use game_cursor::GameCursorPlugin;
use hook::{Hook, HookPlugin};
use light_consts::lux::{AMBIENT_DAYLIGHT, CLEAR_SUNRISE};
use map_state::MapStatePlugin;
use player::PlayerPlugin;
use primary_block::{PrimaryBlock, PrimaryBlockPlugin};
use rock::RockPlugin;
use room::RoomPlugin;

use crate::AppState;

mod camera;
mod game_cursor;
mod hook;
mod map_state;
mod player;
mod primary_block;
mod rock;
mod room;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapStatePlugin)
            .add_plugins(GameCameraPlugin(Vec3::new(0.0, 0.0, 15.0)))
            .add_plugins(PlayerPlugin)
            .add_plugins(GameCursorPlugin)
            .add_plugins(PrimaryBlockPlugin)
            .add_plugins(RoomPlugin)
            .add_plugins(RockPlugin)
            .add_plugins(HookPlugin)
            .add_systems(OnEnter(AppState::Game), setup)
            .add_systems(OnExit(AppState::Game), cleanup)
            .insert_state(GameState::Idle);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let directional_light = |x, y| {
        (
            DirectionalLight {
                illuminance: CLEAR_SUNRISE,
                shadows_enabled: true,
                ..Default::default()
            },
            Transform::from_xyz(x, y, 10.0).looking_at(Vec3::ZERO, Vec3::Z),
        )
    };

    commands.spawn(directional_light(1.0, 1.0));
    commands.spawn(directional_light(1.0, -1.0));
    commands.spawn(directional_light(-1.0, 1.0));
    commands.spawn(directional_light(-1.0, -1.0));

    commands.spawn(PrimaryBlock { x: 0, y: 0 }).with_child(Hook);

    // commands.spawn((
    //     AudioPlayer::new(asset_server.load("Cojam - Milky Main Menu.ogg")),
    //     PlaybackSettings {
    //         mode: PlaybackMode::Loop,
    //         speed: 0.5,
    //         ..Default::default()
    //     },
    // ));
}

fn cleanup() {}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    Idle,
    Pause,
}
