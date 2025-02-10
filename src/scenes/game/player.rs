use bevy::{dev_tools::fps_overlay::FpsOverlayConfig, prelude::*};

use crate::scenes::AppState;

use super::map_state::Structure;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, listen_inputs.run_if(in_state(AppState::Game)))
            .insert_state(PlayerState::Idle);
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
pub enum PlayerState {
    Idle,
    Construct(Structure),
    Destruct,
    Interact,
}

#[derive(Resource)]
pub struct PlayerInteractEntity(pub Entity);

fn listen_inputs(
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayerState>>,
    state: Res<State<PlayerState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    fps_overlay_config: Res<FpsOverlayConfig>,
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
}
