use bevy::prelude::*;

use crate::scenes::AppState;

use super::{builder::Ready, GameState};

pub struct CargoPlugin;

impl Plugin for CargoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update, init)
                .chain()
                .run_if(in_state(AppState::Game).and(in_state(GameState::Idle))),
        );
    }
}

#[derive(Component)]
pub struct Cargo;

#[derive(Component, PartialEq)]
enum CargoState {
    Done,
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&CargoState>), With<Cargo>>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("cargo.glb"))),
                    Ready,
                    CargoState::Done,
                ));
            }
            Some(CargoState::Done) => {}
        }
    }
}

fn update() {}
