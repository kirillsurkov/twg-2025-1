use bevy::{gltf::GltfMaterialName, prelude::*};

use crate::scenes::AppState;

use super::builder::{Enabled, Ready};

pub struct CrusherPlugin;

impl Plugin for CrusherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update, init).chain().run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct Crusher;

#[derive(Component, PartialEq)]
enum CrusherState {
    Materials,
    Done { cylinders: Vec<Entity> },
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&CrusherState>), With<Crusher>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset("crusher.glb")),
                    ),
                    CrusherState::Materials,
                    Visibility::Hidden,
                ));
            }
            Some(CrusherState::Materials) => {
                let mut cylinders = vec![];
                for child in children.iter_descendants(entity) {
                    if gltf_materials
                        .get(child)
                        .map_or(false, |m| m.0 == "Material.014")
                    {
                        cylinders.push(child);
                    }
                }
                if !cylinders.is_empty() {
                    commands
                        .entity(entity)
                        .insert(Ready)
                        .insert(CrusherState::Done { cylinders })
                        .insert(Visibility::Inherited);
                }
            }
            Some(CrusherState::Done { .. }) => {}
        }
    }
}

fn update(
    crushers: Query<&CrusherState, With<Enabled>>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for crusher in crushers.iter() {
        let CrusherState::Done { cylinders } = crusher else {
            continue;
        };

        for cylinder in cylinders {
            if let Ok(mut cylinder) = transforms.get_mut(*cylinder) {
                cylinder.rotate_z(time.delta_secs());
            }
        }
    }
}
