use bevy::{dev_tools::fps_overlay::FpsOverlayConfig, prelude::*};

use crate::scenes::AppState;

use super::{
    game_cursor::GameCursor,
    map_state::{MapLayer, MapState, Structure},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, listen_inputs.run_if(in_state(AppState::Game)))
            .insert_state(PlayerState::Idle);
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PlayerState {
    Idle,
    Construct(Structure),
    Destruct,
    Interact(i32, i32),
}

fn listen_inputs(
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayerState>>,
    state: Res<State<PlayerState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    fps_overlay_config: Res<FpsOverlayConfig>,
    game_cursor: Option<Res<GameCursor>>,
    map_state: Res<MapState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
        match state.get() {
            PlayerState::Idle => {}
            _ => next_state.set(PlayerState::Idle),
        }
    }

    if keyboard.just_pressed(KeyCode::KeyD) {
        match state.get() {
            PlayerState::Destruct => next_state.set(PlayerState::Idle),
            _ => next_state.set(PlayerState::Destruct),
        }
    }

    if keyboard.just_pressed(KeyCode::KeyF) {
        commands.insert_resource(FpsOverlayConfig {
            enabled: !fps_overlay_config.enabled,
            ..Default::default()
        });
    }

    if let Some(game_cursor) = game_cursor {
        if *state.get() == PlayerState::Idle && game_cursor.just_pressed {
            if map_state.node(game_cursor.x, game_cursor.y, MapLayer::Main) {
                next_state.set(PlayerState::Interact(game_cursor.x, game_cursor.y));
            }
        }
    }
}
