use bevy::{gltf::GltfMaterialName, prelude::*};

use crate::scenes::AppState;

use super::builder::Ready;

pub struct FurnacePlugin;

impl Plugin for FurnacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update, init).chain().run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct Furnace;

#[derive(Component, PartialEq)]
enum FurnaceState {
    Materials,
    Done { glass: Entity },
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&FurnaceState>), With<Furnace>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset("furnace.glb")),
                    ),
                    FurnaceState::Materials,
                    Visibility::Hidden,
                ));
            }
            Some(FurnaceState::Materials) => {
                for child in children.iter_descendants(entity) {
                    if gltf_materials
                        .get(child)
                        .map_or(false, |m| m.0 == "Material.008")
                    {
                        commands
                            .entity(entity)
                            .insert(Ready)
                            .insert(FurnaceState::Done { glass: child })
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(FurnaceState::Done { .. }) => {}
        }
    }
}

fn update() {}
