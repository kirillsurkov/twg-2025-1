use bevy::{
    gltf::GltfMaterialName,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
    },
};

use crate::{
    game::game_cursor::GameCursor,
    modify_material::{ModifyMaterial, ModifyMaterialPlugin},
    procedural_material::{
        ProceduralMaterial, ProceduralMaterialPlugin, TextureDef, TextureLayer, TextureMode,
        TextureUpdate,
    },
};

use super::PlayerState;

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RoomFloorMaterial>::default())
            .add_plugins(MaterialPlugin::<
                ExtendedMaterial<StandardMaterial, MyExtension>,
            >::default())
            .add_plugins(ModifyMaterialPlugin::<
                ExtendedMaterial<StandardMaterial, MyExtension>,
            >::default())
            .add_systems(
                Update,
                (
                    (init_room, update_room_pos).chain(),
                    update_room_material,
                    update_room_state,
                    update_floor_material,
                ),
            )
            .add_systems(
                PostUpdate,
                room_builder.run_if(resource_exists::<GameCursor>),
            );
    }
}

#[derive(Component, Reflect)]
enum LoadingState {
    Materials,
    Done,
}

#[derive(Component, Clone, Reflect)]
pub enum RoomState {
    Idle,
    PlayerSelect,
    SelectedForConstruction,
    SelectedForDestruction,
    Construct(f32),
    Destruct,
}

#[derive(Component)]
pub struct Room {
    pub x: i32,
    pub y: i32,
}

fn init_room(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&LoadingState>), With<Room>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                println!("Spawning room");
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb"))),
                    LoadingState::Materials,
                ));
            }
            Some(LoadingState::Materials) => {
                for e in children.iter_descendants(entity) {
                    if gltf_materials.get(e).map_or(false, |m| m.0 == "room_floor") {
                        println!("Found room_floor");
                        commands
                            .entity(e)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(MeshMaterial3d(materials.add(StandardMaterial::default())))
                            .insert(RoomFloorMaterial::new(rand::random::<f32>() * 1000.0));
                        commands.entity(entity).insert(LoadingState::Done);
                    }
                }
            }
            Some(LoadingState::Done) => {}
        }
    }
}

fn update_room_pos(mut rooms: Query<(&Room, &mut Transform)>) {
    for (room, mut transform) in rooms.iter_mut() {
        *transform = Transform::from_xyz(room.x as f32 * 2.01, room.y as f32 * 2.01, 0.0);
    }
}

#[derive(Resource)]
struct RoomBuilderEntity(Entity);

fn room_builder(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut rooms: Query<(&mut Room, &mut RoomState)>,
    game_cursor: Res<GameCursor>,
    room_builder_entity: Option<ResMut<RoomBuilderEntity>>,
    time: Res<Time>,
) {
    let PlayerState::Construct = *player_state else {
        if let Some(entity) = room_builder_entity {
            commands.entity(entity.0).despawn_recursive();
            commands.remove_resource::<RoomBuilderEntity>();
        }
        return;
    };

    let entity = match room_builder_entity {
        Some(entity) => entity.0,
        None => {
            let entity = commands
                .spawn((
                    Room {
                        x: game_cursor.x,
                        y: game_cursor.y,
                    },
                    RoomState::SelectedForConstruction,
                ))
                .id();
            commands.insert_resource(RoomBuilderEntity(entity));
            return;
        }
    };

    let (mut room, mut room_state) = rooms.get_mut(entity).unwrap();
    room.x = game_cursor.x;
    room.y = game_cursor.y;

    if game_cursor.just_pressed {
        *room_state = RoomState::Construct(time.elapsed_secs());
        *player_state = PlayerState::Idle;
        commands.remove_resource::<RoomBuilderEntity>();
    }
}

fn update_room_material(
    mut commands: Commands,
    rooms: Query<
        (Entity, &RoomState, &LoadingState),
        Or<(Changed<RoomState>, Changed<LoadingState>)>,
    >,
) {
    for (entity, room_state, loading_state) in rooms.iter() {
        let LoadingState::Done = *loading_state else {
            continue;
        };
        let mut entity = commands.entity(entity);
        entity.remove::<ModifyMaterial>();
        match *room_state {
            RoomState::SelectedForConstruction => {
                entity.insert(ModifyMaterial::new(|_| StandardMaterial {
                    base_color: Srgba::new(0.0, 10.0, 0.0, 0.01).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                }));
            }
            RoomState::SelectedForDestruction => {
                entity.insert(ModifyMaterial::new(|_| StandardMaterial {
                    base_color: Srgba::new(3.0, 0.0, 0.0, 0.01).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                }));
            }
            RoomState::Construct(created) => {
                entity.insert(ModifyMaterial::new(move |mut mat| ExtendedMaterial {
                    base: {
                        mat.alpha_mode = AlphaMode::Blend;
                        mat
                    },
                    extension: MyExtension { created },
                }));
            }
            _ => {}
        }
    }
}

fn update_room_state(mut rooms: Query<(&mut RoomState, &LoadingState)>, time: Res<Time>) {
    let elapsed = time.elapsed_secs();
    for (mut room_state, loading_state) in rooms.iter_mut() {
        let LoadingState::Done = loading_state else {
            continue;
        };
        match *room_state {
            RoomState::Construct(created) if elapsed - created >= 3.0 => {
                *room_state = RoomState::Idle
            }
            _ => {}
        }
    }
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy)]
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
        (120, 120)
    }
}

fn update_floor_material(mut settings: Query<&mut RoomFloorMaterial>, time: Res<Time>) {
    for mut settings in &mut settings {
        settings.time = time.elapsed_secs();
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
struct MyExtension {
    #[uniform(100)]
    created: f32,
}

impl MaterialExtension for MyExtension {
    fn fragment_shader() -> ShaderRef {
        "build.wgsl".into()
    }
}
