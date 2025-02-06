use bevy::{gltf::GltfMaterialName, prelude::*};

use super::{map_state::MapState, room::RoomFloorMaterial};

pub struct PrimaryBlockPlugin;

impl Plugin for PrimaryBlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init_primary_block);
    }
}

#[derive(Component)]
pub struct PrimaryBlock {
    pub x: i32,
    pub y: i32,
}

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut room_locations: ResMut<MapState>,
) {
    for (entity, primary_block, state) in primary_blocks.iter() {
        match state {
            None => {
                commands
                    .entity(entity)
                    .insert(SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb")),
                    ))
                    // .insert(Mesh3d(meshes.add(Cuboid::from_length(2.0))))
                    // .insert(MeshMaterial3d(materials.add(StandardMaterial::default())))
                    .insert(Transform::from_xyz(
                        primary_block.x as f32 * 2.01,
                        primary_block.y as f32 * 2.01,
                        0.0,
                    ))
                    .insert(LoadingState::Materials);
                room_locations.add_primary_block(primary_block.x, primary_block.y);
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
                            .insert(LoadingState::Done)
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(LoadingState::Done) => {}
        }
    }
}
