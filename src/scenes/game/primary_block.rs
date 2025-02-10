use bevy::{gltf::GltfMaterialName, prelude::*};

use crate::scenes::AppState;

use super::{builder::Ready, map_state::MapState, room::RoomFloorMaterial};

pub struct PrimaryBlockPlugin;

impl Plugin for PrimaryBlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init_primary_block.run_if(in_state(AppState::Game)));
    }
}

#[derive(Component)]
pub struct PrimaryBlock;

#[derive(Component, Reflect)]
enum LoadingState {
    Materials,
    Done,
}

fn init_primary_block(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    primary_blocks: Query<(Entity, &PrimaryBlock, Option<&LoadingState>)>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, primary_block, state) in primary_blocks.iter() {
        match state {
            None => {
                commands
                    .entity(entity)
                    .insert(SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb")),
                    ))
                    .insert(LoadingState::Materials);
            }
            Some(LoadingState::Materials) => {
                for child in children.iter_descendants(entity) {
                    if gltf_materials
                        .get(child)
                        .map_or(false, |m| m.0 == "room_floor")
                    {
                        commands
                            .entity(child)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(RoomFloorMaterial::new(
                                rand::random::<f32>() * 1000.0,
                                0.0,
                                1.0,
                            ));
                        commands
                            .entity(entity)
                            .insert(Ready)
                            .insert(LoadingState::Done)
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(LoadingState::Done) => {}
        }
    }
}
