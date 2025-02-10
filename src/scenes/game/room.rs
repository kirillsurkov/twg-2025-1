use bevy::{gltf::GltfMaterialName, prelude::*, render::render_resource::ShaderType};
use bevy_rapier2d::prelude::*;

use crate::{
    components::procedural_material::{ProceduralMaterial, ProceduralMaterialPlugin},
    scenes::AppState,
};

use super::{
    builder::Ready,
    game_cursor::{CursorLayer, GameCursor},
    player::PlayerState,
};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RoomFloorMaterial>::default())
            .add_systems(
                Update,
                (update_floor_material, (update, init).chain()).run_if(in_state(AppState::Game)),
            );
    }
}

#[derive(Component)]
pub struct Room;

#[derive(Component, PartialEq)]
enum RoomState {
    Materials,
    Done { floor: Entity },
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&RoomState>), With<Room>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb"))),
                    RoomState::Materials,
                    Visibility::Hidden,
                    Collider::cuboid(1.0, 1.0),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::STATIC_STATIC,
                ));
            }
            Some(RoomState::Materials) => {
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
                                0.95,
                                1.0,
                            ));
                        commands
                            .entity(entity)
                            .insert(Ready)
                            .insert(RoomState::Done { floor: child })
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(RoomState::Done { .. }) => {}
        }
    }
}

fn update(
    mut rooms: Query<(&RoomState, &Transform), With<Room>>,
    mut floor_materials: Query<&mut RoomFloorMaterial>,
    game_cursor: Option<Res<GameCursor>>,
    player_state: Res<State<PlayerState>>,
) {
    for (loading_state, transform) in rooms.iter_mut() {
        let floor = match loading_state {
            RoomState::Done { floor } => floor,
            _ => continue,
        };

        let Vec2 { x, y } = transform.translation.xy();
        let IVec2 { x, y } = GameCursor::world_to_game(x, y, CursorLayer::Room);

        let is_selected = match game_cursor.as_ref() {
            Some(game_cursor) => x == game_cursor.x && y == game_cursor.y,
            _ => false,
        };

        if *player_state.get() == PlayerState::Idle {
            let mut floor_material = floor_materials.get_mut(*floor).unwrap();
            floor_material.time_multiplier = if is_selected { 100.0 } else { 1.0 };
        }
    }
}

fn update_floor_material(mut settings: Query<&mut RoomFloorMaterial>, time: Res<Time>) {
    for mut settings in settings.iter_mut() {
        settings.time += time.delta_secs() * settings.time_multiplier;
    }
}

#[derive(Component, ShaderType, Clone)]
pub struct RoomFloorMaterial {
    seed: f32,
    time: f32,
    time_multiplier: f32,
    low_edge: f32,
    high_edge: f32,
}

impl RoomFloorMaterial {
    pub fn new(seed: f32, low_edge: f32, high_edge: f32) -> Self {
        Self {
            seed,
            time: 0.0,
            time_multiplier: 1.0,
            low_edge,
            high_edge,
        }
    }
}

impl ProceduralMaterial for RoomFloorMaterial {
    fn shader() -> &'static str {
        "room_floor.wgsl"
    }

    fn size() -> (u32, u32) {
        (36, 36)
    }
}
