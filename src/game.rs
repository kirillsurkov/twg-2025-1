use bevy::{prelude::*, utils::hashbrown::HashSet};
use camera::GameCameraPlugin;
use game_cursor::GameCursorPlugin;
use light_consts::lux::AMBIENT_DAYLIGHT;
use primary_block::{PrimaryBlock, PrimaryBlockPlugin};
use room::RoomPlugin;

mod camera;
mod game_cursor;
mod primary_block;
mod room;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameCameraPlugin(Vec3::new(0.0, 0.0, 15.0)))
            .add_plugins(PrimaryBlockPlugin)
            .add_plugins(RoomPlugin)
            .add_plugins(GameCursorPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update_state)
            .insert_resource(RoomLocations::default())
            .insert_resource(PlayerState::Idle);
    }
}

#[derive(Resource, Default)]
pub struct RoomLocations {
    available: HashSet<IVec2>,
    unavailable: HashSet<IVec2>,
}

impl RoomLocations {
    fn validate_and_insert(&mut self, x: i32, y: i32) {
        let value = IVec2::new(x, y);
        if !self.unavailable.contains(&value) {
            self.available.insert(value);
        }
    }

    pub fn insert_around(&mut self, x: i32, y: i32) {
        self.validate_and_insert(x, y + 1);
        self.validate_and_insert(x, y - 1);
        self.validate_and_insert(x + 1, y);
        self.validate_and_insert(x - 1, y);
        let value = IVec2::new(x, y);
        self.available.remove(&value);
        self.unavailable.insert(value);
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
