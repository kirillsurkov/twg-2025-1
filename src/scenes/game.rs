use bevy::prelude::*;
use camera::GameCameraPlugin;
use game_cursor::GameCursorPlugin;
use hook::{Hook, HookPlugin};
use light_consts::lux::CLEAR_SUNRISE;
use map_state::MapStatePlugin;
use player::PlayerPlugin;
use primary_block::{PrimaryBlock, PrimaryBlockPlugin};
use rock::RockPlugin;
use room::RoomPlugin;

use super::{AppSceneRoot, AppState};

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
            .insert_state(GameState::Idle);
    }
}

fn setup(mut commands: Commands, root_entity: Res<AppSceneRoot>) {
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

    commands.entity(root_entity.world).with_children(|root| {
        root.spawn(directional_light(1.0, 1.0));
        root.spawn(directional_light(1.0, -1.0));
        root.spawn(directional_light(-1.0, 1.0));
        root.spawn(directional_light(-1.0, -1.0));
        root.spawn(PrimaryBlock { x: 0, y: 0 }).with_child(Hook);
    });

    // commands.spawn((
    //     AudioPlayer::new(asset_server.load("Cojam - Milky Main Menu.ogg")),
    //     PlaybackSettings {
    //         mode: PlaybackMode::Loop,
    //         speed: 0.5,
    //         ..Default::default()
    //     },
    // ));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    Idle,
    Pause,
}
