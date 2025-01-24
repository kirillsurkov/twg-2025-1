use bevy::{gltf::GltfMaterialName, prelude::*};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (init_room, update_room));
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
                        commands.entity(room).insert(State::Done);
                    }
                }
            }
            Some(State::Done) => {}
        }
    }
}

fn update_room() {}
