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
        ExtendedProceduralMaterial, ProceduralMaterial, ProceduralMaterialPlugin,
    },
};

use super::{PlayerState, RoomLocations};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RoomFloorMaterial>::default())
            .add_plugins(MaterialPlugin::<ExtendedBuildMaterial>::default())
            .add_plugins(ModifyMaterialPlugin::<
                StandardMaterial,
                ExtendedBuildMaterial,
            >::default())
            .add_plugins(ModifyMaterialPlugin::<
                ExtendedProceduralMaterial,
                ExtendedBuildMaterial,
            >::default())
            .add_systems(
                Update,
                (
                    (update_room_material, init_room, update_room_pos).chain(),
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

#[derive(Component)]
pub enum Room {
    Fixed(i32, i32),
    Floating(f32, f32),
}

#[derive(Component, Clone, Reflect, PartialEq)]
pub enum RoomState {
    Idle,
    PlayerSelect,
    SelectedForConstruction,
    SelectedForDestruction,
    Construct(f32),
    Destruct,
}

#[derive(Component, Reflect)]
enum LoadingState {
    Materials,
    Done,
}

fn init_room(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rooms: Query<(Entity, Option<&LoadingState>), With<Room>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    for (entity, state) in rooms.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("room.glb"))),
                    LoadingState::Materials,
                    Visibility::Hidden,
                ));
            }
            Some(LoadingState::Materials) => {
                for e in children.iter_descendants(entity) {
                    if gltf_materials.get(e).map_or(false, |m| m.0 == "room_floor") {
                        commands
                            .entity(e)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(RoomFloorMaterial::new(rand::random::<f32>() * 1000.0));
                        commands
                            .entity(entity)
                            .insert(LoadingState::Done)
                            .remove::<Visibility>()
                            .insert(Visibility::Inherited);
                    }
                }
            }
            Some(LoadingState::Done) => {}
        }
    }
}

fn update_room_pos(mut rooms: Query<(&Room, &mut Transform)>) {
    for (room, mut transform) in rooms.iter_mut() {
        let (x, y, z) = match *room {
            Room::Fixed(x, y) => (x as f32 * 2.01, y as f32 * 2.01, 0.0),
            Room::Floating(x, y) => (x, y, 0.1),
        };
        *transform = Transform::from_xyz(x, y, z);
    }
}

#[derive(Resource)]
struct RoomBuilderEntity(Entity);

fn room_builder(
    mut commands: Commands,
    mut room_locations: ResMut<RoomLocations>,
    mut player_state: ResMut<PlayerState>,
    mut rooms: Query<(&mut Room, &mut RoomState)>,
    game_cursor: Res<GameCursor>,
    room_builder_entity: Option<Res<RoomBuilderEntity>>,
    time: Res<Time>,
) {
    let PlayerState::Construct = *player_state else {
        if let Some(entity) = room_builder_entity {
            commands.entity(entity.0).despawn_recursive();
            commands.remove_resource::<RoomBuilderEntity>();
        }
        return;
    };

    let Some(entity) = room_builder_entity.map(|e| e.0) else {
        let entity = commands
            .spawn((
                Room::Fixed(game_cursor.x, game_cursor.y),
                RoomState::SelectedForConstruction,
            ))
            .id();
        commands.insert_resource(RoomBuilderEntity(entity));
        return;
    };

    let (mut room, mut room_state) = rooms.get_mut(entity).unwrap();

    let available = room_locations
        .available
        .contains(&IVec2::new(game_cursor.x, game_cursor.y));

    if available {
        *room = Room::Fixed(game_cursor.x, game_cursor.y);
        if *room_state != RoomState::SelectedForConstruction {
            *room_state = RoomState::SelectedForConstruction;
        }
    } else {
        *room = Room::Floating(game_cursor.fx, game_cursor.fy);
        if *room_state != RoomState::SelectedForDestruction {
            *room_state = RoomState::SelectedForDestruction;
        }
    }

    if available && game_cursor.just_pressed {
        *room_state = RoomState::Construct(time.elapsed_secs());
        *player_state = PlayerState::Idle;
        commands.remove_resource::<RoomBuilderEntity>();
        room_locations.insert_around(game_cursor.x, game_cursor.y);
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
        entity
            .remove::<ModifyMaterial<StandardMaterial>>()
            .remove::<ModifyMaterial<ExtendedProceduralMaterial>>();
        match *room_state {
            RoomState::SelectedForConstruction => {
                let material = StandardMaterial {
                    base_color: Srgba::new(0.0, 10.0, 0.0, 0.01).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                };
                entity
                    .insert(ModifyMaterial::new({
                        let material = material.clone();
                        move |_: StandardMaterial| material.clone()
                    }))
                    .insert(ModifyMaterial::new({
                        let material = material.clone();
                        move |_: ExtendedProceduralMaterial| material.clone()
                    }));
            }
            RoomState::SelectedForDestruction => {
                let material = StandardMaterial {
                    base_color: Srgba::new(3.0, 0.0, 0.0, 0.01).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                };
                entity
                    .insert(ModifyMaterial::new({
                        let material = material.clone();
                        move |_: StandardMaterial| material.clone()
                    }))
                    .insert(ModifyMaterial::new({
                        let material = material.clone();
                        move |_: ExtendedProceduralMaterial| material.clone()
                    }));
            }
            RoomState::Construct(created) => {
                let extension = BuildMaterial { created };
                entity
                    .insert(ModifyMaterial::new({
                        let extension = extension.clone();
                        move |mut mat: StandardMaterial| ExtendedMaterial {
                            base: {
                                mat.alpha_mode = AlphaMode::Blend;
                                mat
                            },
                            extension: extension.clone(),
                        }
                    }))
                    .insert(ModifyMaterial::new({
                        let extension = extension.clone();
                        move |_: ExtendedProceduralMaterial| ExtendedMaterial {
                            base: StandardMaterial {
                                base_color: Color::BLACK,
                                alpha_mode: AlphaMode::Blend,
                                ..Default::default()
                            },
                            extension: extension.clone(),
                        }
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
        (36, 36)
    }
}

fn update_floor_material(mut settings: Query<&mut RoomFloorMaterial>, time: Res<Time>) {
    for mut settings in &mut settings {
        settings.time = time.elapsed_secs();
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct BuildMaterial {
    #[uniform(100)]
    created: f32,
}

type ExtendedBuildMaterial = ExtendedMaterial<StandardMaterial, BuildMaterial>;

impl MaterialExtension for BuildMaterial {
    fn fragment_shader() -> ShaderRef {
        "build.wgsl".into()
    }
}
