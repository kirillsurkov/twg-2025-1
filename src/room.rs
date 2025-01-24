use bevy::{
    gltf::GltfMaterialName,
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

use crate::procedural_material::{
    ProceduralMaterial, ProceduralMaterialPlugin, TextureDef, TextureLayer, TextureMode,
    TextureUpdate,
};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RoomFloorMaterial>::default())
            .add_systems(Update, (init_room, update_room, update_floor_material));
    }
}

#[derive(Component, Reflect)]
enum State {
    Materials,
    Done,
}

#[derive(Component)]
pub struct Room;

fn init_room(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&State>), With<Room>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (room, state) in rooms.iter() {
        match state {
            None => {
                println!("Spawning room");
                commands.entity(room).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb"))),
                    State::Materials,
                ));
            }
            Some(State::Materials) => {
                for e in children.iter_descendants(room) {
                    if gltf_materials.get(e).map_or(false, |m| m.0 == "room_floor") {
                        println!("Found room_floor");
                        commands
                            .entity(e)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(MeshMaterial3d(materials.add(StandardMaterial::default())))
                            .insert(RoomFloorMaterial::new(rand::random::<f32>() * 1000.0));
                        commands.entity(room).insert(State::Done);
                    }
                }
            }
            Some(State::Done) => {}
        }
    }
}

fn update_room() {}

#[derive(Component, ExtractComponent, ShaderType, Default, Clone, Copy)]
struct RoomFloorMaterial {
    time: f32,
    seed: f32,
}

impl RoomFloorMaterial {
    fn new(seed: f32) -> Self {
        Self { time: 0.0, seed }
    }
}

impl ProceduralMaterial for RoomFloorMaterial {
    fn shader() -> &'static str {
        "room_floor.wgsl"
    }

    fn size() -> (u32, u32) {
        (512, 512)
    }

    fn texture_def(layer: TextureLayer) -> TextureDef {
        match layer {
            TextureLayer::Emissive => TextureDef {
                mode: TextureMode::Private,
                update: TextureUpdate::EachFrame,
            },
            _ => TextureDef::default(),
        }
    }
}

fn update_floor_material(mut settings: Query<&mut RoomFloorMaterial>, time: Res<Time>) {
    for mut settings in &mut settings {
        settings.time = time.elapsed_secs();
    }
}
