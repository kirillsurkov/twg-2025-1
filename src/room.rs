use bevy::{
    gltf::GltfMaterialName,
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

use crate::{
    modify_material::ModifyMaterial,
    procedural_material::{
        ProceduralMaterial, ProceduralMaterialPlugin, TextureDef, TextureLayer, TextureMode,
        TextureUpdate,
    },
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
pub struct Room {
    pub x: i32,
    pub y: i32,
}

fn on_over(root: Entity, index: u32) -> impl Fn(Trigger<Pointer<Over>>, Commands) {
    move |_, mut commands| {
        commands
            .entity(root)
            .insert(ModifyMaterial::new(move |mut material| {
                if index == 0 {
                    material.diffuse_transmission = -1.0;
                    material.specular_transmission = -1.0;
                    material
                } else if index == 1 {
                    StandardMaterial {
                        base_color: Srgba::new(0.5, 3.0, 0.5, 0.2).into(),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    }
                } else if index == 2 {
                    StandardMaterial {
                        base_color: Srgba::new(3.0, 0.2, 0.2, 0.2).into(),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..Default::default()
                    }
                } else {
                    material
                }
            }));
    }
}

fn on_out(root: Entity) -> impl Fn(Trigger<Pointer<Out>>, Commands) {
    move |_, mut commands| {
        commands.entity(root).remove::<ModifyMaterial>();
    }
}

fn init_room(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, &Room, Option<&State>)>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    let mut index = 0;
    for (entity, room, state) in rooms.iter() {
        match state {
            None => {
                println!("Spawning room");
                commands
                    .entity(entity)
                    .insert((
                        SceneRoot(
                            asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb")),
                        ),
                        Transform::from_xyz(room.x as f32 * 2.01, room.y as f32 * 2.01, 0.0),
                        State::Materials,
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
                                MeshMaterial3d(materials.add(StandardMaterial::default())),
                                Visibility::Hidden,
                                RayCastPickable,
                            ))
                            .observe(on_over(entity, index))
                            .observe(on_out(entity));
                    });
                index += 1;
            }
            Some(State::Materials) => {
                for e in children.iter_descendants(entity) {
                    if gltf_materials.get(e).map_or(false, |m| m.0 == "room_floor") {
                        println!("Found room_floor");
                         commands
                             .entity(e)
                             .remove::<MeshMaterial3d<StandardMaterial>>()
                             .insert(MeshMaterial3d(materials.add(StandardMaterial::default())))
                             .insert(RoomFloorMaterial::new(rand::random::<f32>() * 1000.0));
                        commands.entity(entity).insert(State::Done);
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
