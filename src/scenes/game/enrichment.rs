use bevy::prelude::*;

use crate::scenes::AppState;

use super::{builder::Ready, GameState};

pub struct EnrichmentPlugin;

impl Plugin for EnrichmentPlugin {
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
pub struct Enrichment;

#[derive(Component, PartialEq)]
enum EnrichmentState {
    Done,
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&EnrichmentState>), With<Enrichment>>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset("enrichment.glb")),
                    ),
                    Ready,
                    EnrichmentState::Done,
                ));
            }
            Some(EnrichmentState::Done) => {}
        }
    }
}

fn update() {}
