use bevy::{gltf::GltfMaterialName, prelude::*, render::render_resource::ShaderType};

use crate::procedural_material::{ProceduralMaterial, ProceduralMaterialPlugin};

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RockMaterial>::default())
            .add_systems(Update, (init, update_pos));
    }
}

#[derive(Component)]
pub struct Rock {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, PartialEq)]
enum LoadingState {
    Materials,
    Done { floor: Entity },
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rocks: Query<(Entity, Option<&LoadingState>), With<Rock>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rocks.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("rock_0.glb"))),
                    LoadingState::Materials,
                    Visibility::Hidden,
                ));
            }
            Some(LoadingState::Materials) => {
                for child in children.iter_descendants(entity) {
                    if gltf_materials
                        .get(child)
                        .map_or(false, |m| m.0 == "rock_material")
                    {
                        commands
                            .entity(child)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(RockMaterial::new(rand::random::<f32>() * 1000.0));
                        commands
                            .entity(entity)
                            .insert(LoadingState::Done { floor: child })
                            .remove::<Visibility>()
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(LoadingState::Done { .. }) => {}
        }
    }
}

fn update_pos(mut rooms: Query<(&Rock, &mut Transform)>) {
    for (room, mut transform) in rooms.iter_mut() {
        *transform = Transform::from_xyz(room.x, room.y, 0.0);
    }
}

#[derive(Component, ShaderType, Clone)]
struct RockMaterial {
    seed: f32,
    time: f32,
    time_multiplier: f32,
}

impl RockMaterial {
    fn new(seed: f32) -> Self {
        Self {
            seed,
            time: 0.0,
            time_multiplier: 1.0,
        }
    }
}

impl ProceduralMaterial for RockMaterial {
    fn shader() -> &'static str {
        "rock.wgsl"
    }

    fn size() -> (u32, u32) {
        (128, 128)
    }
}
