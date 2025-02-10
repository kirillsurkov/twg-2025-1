use bevy::{gltf::GltfMaterialName, prelude::*};

use crate::scenes::AppState;

use super::builder::Ready;

pub struct GeneratorPlugin;

impl Plugin for GeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update, init).chain().run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct Generator;

#[derive(Component, PartialEq)]
enum GeneratorState {
    Materials,
    Done { ball: Entity },
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&GeneratorState>), With<Generator>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset("generator.glb")),
                    ),
                    GeneratorState::Materials,
                    Visibility::Hidden,
                ));
            }
            Some(GeneratorState::Materials) => {
                for child in children.iter_descendants(entity) {
                    if gltf_materials
                        .get(child)
                        .map_or(false, |m| m.0 == "Material.019")
                    {
                        commands
                            .entity(entity)
                            .insert(Ready)
                            .insert(GeneratorState::Done { ball: child })
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(GeneratorState::Done { .. }) => {}
        }
    }
}

fn update() {}
