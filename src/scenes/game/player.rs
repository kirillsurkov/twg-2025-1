use bevy::{dev_tools::fps_overlay::FpsOverlayConfig, prelude::*};

use crate::scenes::AppState;

use super::{
    game_cursor::GameCursor,
    map_state::{MapLayer, MapNode, MapState},
    GameState,
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
    Construct(MapNode),
    Destruct,
    Interact(i32, i32),
}

fn listen_inputs(
    mut commands: Commands,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    game_state: Res<State<GameState>>,
    player_state: Res<State<PlayerState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    // fps_overlay_config: Res<FpsOverlayConfig>,
    game_cursor: Option<Res<GameCursor>>,
    map_state: Res<MapState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match game_state.get() {
            GameState::Pause => next_game_state.set(GameState::Idle),
            GameState::Idle => next_game_state.set(GameState::Pause),
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        next_player_state.set(PlayerState::Idle);
    }

    if keyboard.just_pressed(KeyCode::KeyD) {
        match player_state.get() {
            PlayerState::Destruct => next_player_state.set(PlayerState::Idle),
            _ => next_player_state.set(PlayerState::Destruct),
        }
    }

    // if keyboard.just_pressed(KeyCode::KeyF) {
    //     commands.insert_resource(FpsOverlayConfig {
    //         enabled: !fps_overlay_config.enabled,
    //         ..Default::default()
    //     });
    // }

    if let Some(game_cursor) = game_cursor {
        if *player_state.get() == PlayerState::Idle && game_cursor.just_pressed {
            if map_state.is_node(game_cursor.x, game_cursor.y, MapLayer::Main) {
                next_player_state.set(PlayerState::Interact(game_cursor.x, game_cursor.y));
            }
        }
    }
}
