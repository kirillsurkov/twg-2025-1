use bevy::{
    gltf::GltfMaterialName,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};
use bevy_rapier2d::prelude::*;

use crate::{
    components::{
        material_modifier::{MaterialModifier, MaterialModifierPlugin},
        procedural_material::{
            ExtendedProceduralMaterial, ProceduralMaterial, ProceduralMaterialPlugin,
        },
    },
    scenes::{AppSceneRoot, AppState},
};

use super::{
    game_cursor::{CursorLayer, GameCursor},
    hook::Hook,
    map_state::{MapLayer, MapState},
    player::PlayerState,
};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RoomFloorMaterial>::default())
            .add_plugins(MaterialPlugin::<ExtendedBuildMaterial>::default())
            .add_plugins(MaterialModifierPlugin::<
                StandardMaterial,
                ExtendedBuildMaterial,
            >::default())
            .add_systems(
                Update,
                (
                    (
                        update_floor_material,
                        (
                            update_room_state,
                            update_room_material,
                            init_room,
                            update_room_pos,
                        )
                            .chain(),
                    ),
                    (
                        transit_state,
                        (
                            state_idle.run_if(in_state(PlayerState::Idle)),
                            state_construct.run_if(in_state(PlayerState::Construct)),
                            state_destruct.run_if(in_state(PlayerState::Destruct)),
                        ),
                    )
                        .chain(),
                )
                    .chain()
                    .run_if(in_state(AppState::Game)),
            );
    }
}

#[derive(Component)]
pub enum Room {
    Fixed(i32, i32),
    Floating(f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightState {
    None,
    White,
    Green,
    Orange,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionState {
    Idle,
    Construct(f32),
    Destruct(f32),
}

#[derive(Component, Clone, PartialEq)]
pub struct RoomState {
    highlight: HighlightState,
    action: ActionState,
}

#[derive(Component, PartialEq)]
enum LoadingState {
    Materials,
    Done { floor: Entity },
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
                    Collider::cuboid(1.0, 1.0),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::STATIC_STATIC,
                ));
            }
            Some(LoadingState::Materials) => {
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

fn update_room_pos(mut rooms: Query<(&Room, &mut Transform)>) {
    for (room, mut transform) in rooms.iter_mut() {
        let (x, y, z) = match *room {
            Room::Fixed(x, y) => {
                let fvec = GameCursor::game_to_world(x, y, CursorLayer::Room);
                (fvec.x, fvec.y, 0.0)
            }
            Room::Floating(fx, fy, fz) => (fx, fy, fz),
        };
        *transform = Transform::from_xyz(x, y, z);
    }
}

#[derive(Resource)]
struct RoomBuildEntity(Entity);

fn transit_state(
    mut commands: Commands,
    mut rooms: Query<(Entity, &Room, &LoadingState, &mut RoomState)>,
    mut floor_materials: Query<&mut RoomFloorMaterial>,
    mut transition_events: EventReader<StateTransitionEvent<PlayerState>>,
    build_entity: Option<Res<RoomBuildEntity>>,
) {
    for event in transition_events.read() {
        match event.exited {
            Some(PlayerState::Idle) => {
                for (_, _, loading_state, mut room_state) in rooms.iter_mut() {
                    if let LoadingState::Done { floor } = loading_state {
                        floor_materials.get_mut(*floor).unwrap().time_multiplier = 1.0;
                    }
                    room_state.highlight = HighlightState::None;
                }
            }
            Some(PlayerState::Construct) => {
                if let Some(RoomBuildEntity(entity)) = build_entity.as_deref() {
                    commands.entity(*entity).despawn_recursive();
                    commands.remove_resource::<RoomBuildEntity>();
                }
            }
            Some(PlayerState::Destruct) => {
                for (_, _, _, mut room_state) in rooms.iter_mut() {
                    room_state.highlight = HighlightState::None;
                }
            }
            _ => {}
        }
    }
}

fn state_idle(
    game_cursor: Option<Res<GameCursor>>,
    mut rooms: Query<(&Room, &LoadingState, &mut RoomState)>,
    mut floor_materials: Query<&mut RoomFloorMaterial>,
) {
    for (room, loading_state, mut room_state) in rooms.iter_mut() {
        let LoadingState::Done { floor } = loading_state else {
            continue;
        };

        let ActionState::Idle = room_state.action else {
            continue;
        };

        let (x, y) = match *room {
            Room::Fixed(x, y) => (x, y),
            Room::Floating(..) => continue,
        };

        let is_selected = match game_cursor.as_ref() {
            Some(game_cursor) => x == game_cursor.x && y == game_cursor.y,
            _ => false,
        };

        let mut floor_material = floor_materials.get_mut(*floor).unwrap();

        if is_selected && room_state.highlight == HighlightState::None {
            room_state.highlight = HighlightState::White;
            floor_material.time_multiplier = 100.0;
        }
        if !is_selected && room_state.highlight == HighlightState::White {
            room_state.highlight = HighlightState::None;
            floor_material.time_multiplier = 1.0;
        }
    }
}

fn state_construct(
    mut commands: Commands,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    mut map_state: ResMut<MapState>,
    mut rooms: Query<(&mut Room, &mut RoomState)>,
    root_entity: Res<AppSceneRoot>,
    game_cursor: Option<Res<GameCursor>>,
    room_interaction: Option<Res<RoomBuildEntity>>,
    time: Res<Time>,
) {
    let Some(game_cursor) = game_cursor.as_ref() else {
        if let Some(RoomBuildEntity(entity)) = room_interaction.as_deref() {
            commands.entity(*entity).despawn_recursive();
            commands.remove_resource::<RoomBuildEntity>();
        }
        return;
    };

    let Some(RoomBuildEntity(entity)) = room_interaction.as_deref() else {
        let entity = commands
            .spawn((
                Room::Fixed(game_cursor.x, game_cursor.y),
                RoomState {
                    action: ActionState::Idle,
                    highlight: HighlightState::Green,
                },
            ))
            .id();
        commands.entity(root_entity.world).add_child(entity);
        commands.insert_resource(RoomBuildEntity(entity));
        return;
    };

    let Ok((mut room, mut room_state)) = rooms.get_mut(*entity) else {
        return;
    };

    let available = map_state.is_available(game_cursor.x, game_cursor.y);

    if available {
        *room = Room::Fixed(game_cursor.x, game_cursor.y);
        if room_state.highlight != HighlightState::Green {
            room_state.highlight = HighlightState::Green;
        }
    } else {
        *room = Room::Floating(game_cursor.fx, game_cursor.fy, 0.1);
        if room_state.highlight != HighlightState::Red {
            room_state.highlight = HighlightState::Red;
        }
    }

    if available && game_cursor.just_pressed {
        commands.entity(*entity).with_child(Hook(true));
        room_state.action = ActionState::Construct(time.elapsed_secs());
        room_state.highlight = HighlightState::None;
        next_player_state.set(PlayerState::Idle);
        commands.remove_resource::<RoomBuildEntity>();
        map_state.add_room(game_cursor.x, game_cursor.y);
    }
}

fn state_destruct(
    mut next_player_state: ResMut<NextState<PlayerState>>,
    mut map_state: ResMut<MapState>,
    mut rooms: Query<(&Room, &mut RoomState)>,
    game_cursor: Option<Res<GameCursor>>,
) {
    for (room, mut room_state) in rooms.iter_mut() {
        let (x, y) = match *room {
            Room::Fixed(x, y) => (x, y),
            Room::Floating(..) => continue,
        };

        if map_state.room(x, y, MapLayer::Build) {
            room_state.highlight = HighlightState::None;
            continue;
        }

        let is_selected = match game_cursor.as_ref() {
            Some(game_cursor) => {
                if game_cursor.just_pressed {
                    map_state.remove(x, y, MapLayer::Main);
                    next_player_state.set(PlayerState::Idle);
                }
                x == game_cursor.x && y == game_cursor.y
            }
            None => false,
        };

        if is_selected && room_state.highlight != HighlightState::Red {
            room_state.highlight = HighlightState::Red;
        }
        if !is_selected && room_state.highlight != HighlightState::Orange {
            room_state.highlight = HighlightState::Orange;
        }
    }

    map_state.sync_build();
    if let Some(game_cursor) = game_cursor.as_ref() {
        map_state.remove(game_cursor.x, game_cursor.y, MapLayer::Build);
    }
}

fn update_room_material(
    mut commands: Commands,
    rooms: Query<(Entity, &RoomState, &LoadingState)>,
    time: Res<Time>,
) {
    for (entity, room_state, loading_state) in rooms.iter() {
        match *loading_state {
            LoadingState::Done { .. } => {}
            _ => continue,
        }

        let mut entity = commands.entity(entity);

        entity
            .remove::<MaterialModifier<StandardMaterial>>()
            .remove::<MaterialModifier<ExtendedBuildMaterial>>()
            .remove::<MaterialModifier<ExtendedProceduralMaterial>>();

        let highlight_white = LinearRgba::WHITE * 0.002;
        let highlight_green = LinearRgba::new(0.0, 1.0, 0.0, 1.0);
        let highlight_orange = LinearRgba::new(1.0, 0.5, 0.0, 1.0);
        let highlight_red = LinearRgba::new(1.0, 0.0, 0.0, 1.0);

        let with_highlight = |mut mat: StandardMaterial, highlight| {
            mat.base_color.set_alpha(0.01);
            mat.emissive = highlight * 10000.0;
            mat.alpha_mode = AlphaMode::Blend;
            mat
        };

        let floor_inactive = MaterialModifier::new(move |mut mat: ExtendedProceduralMaterial| {
            mat.base.base_color.set_alpha(0.5);
            mat.base.emissive = LinearRgba::NONE;
            mat
        });

        match (room_state.action, room_state.highlight) {
            (ActionState::Idle, HighlightState::None) => {}
            (ActionState::Idle, HighlightState::White) => {
                entity
                    .insert(MaterialModifier::new(move |mut mat: StandardMaterial| {
                        mat.emissive += highlight_white;
                        mat.alpha_mode = AlphaMode::Blend;
                        mat
                    }))
                    .insert(MaterialModifier::new(
                        move |mut mat: ExtendedProceduralMaterial| {
                            mat.extension.add_emission = highlight_white;
                            mat
                        },
                    ));
            }
            (ActionState::Idle, HighlightState::Green) => {
                entity.insert(floor_inactive).insert(MaterialModifier::new(
                    move |mat: StandardMaterial| with_highlight(mat, highlight_green),
                ));
            }
            (ActionState::Idle, HighlightState::Orange) => {
                entity.insert(floor_inactive).insert(MaterialModifier::new(
                    move |mat: StandardMaterial| with_highlight(mat, highlight_orange),
                ));
            }
            (ActionState::Idle, HighlightState::Red) => {
                entity.insert(floor_inactive).insert(MaterialModifier::new(
                    move |mat: StandardMaterial| with_highlight(mat, highlight_red),
                ));
            }
            (ActionState::Construct(created), highlight) => {
                entity
                    .insert(MaterialModifier::new({
                        move |mut mat: StandardMaterial| ExtendedMaterial {
                            base: {
                                mat.alpha_mode = AlphaMode::Blend;
                                mat
                            },
                            extension: BuildMaterial {
                                settings: BuildMaterialSettings {
                                    created,
                                    color: match highlight {
                                        HighlightState::Orange => highlight_orange,
                                        HighlightState::Red => highlight_red,
                                        _ => highlight_green,
                                    },
                                    direction: 1.0,
                                },
                            },
                        }
                    }))
                    .insert(floor_inactive);
            }
            (ActionState::Destruct(created), _) => {
                entity.insert(MaterialModifier::new({
                    move |mut mat: StandardMaterial| ExtendedMaterial {
                        base: {
                            mat.alpha_mode = AlphaMode::Blend;
                            mat
                        },
                        extension: BuildMaterial {
                            settings: BuildMaterialSettings {
                                created,
                                color: highlight_red,
                                direction: -1.0,
                            },
                        },
                    }
                }));
            }
        }
    }
}

fn update_floor_material(mut settings: Query<&mut RoomFloorMaterial>, time: Res<Time>) {
    for mut settings in &mut settings {
        settings.time += time.delta_secs() * settings.time_multiplier;
    }
}

fn update_room_state(
    mut commands: Commands,
    mut rooms: Query<(Entity, &mut Room, &LoadingState, &mut RoomState)>,
    build_entity: Option<Res<RoomBuildEntity>>,
    map_state: Res<MapState>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();
    let delta = time.delta_secs();
    for (entity, mut room, loading_state, mut room_state) in rooms.iter_mut() {
        match loading_state {
            LoadingState::Done { .. } => {}
            _ => continue,
        }
        if let Room::Fixed(x, y) = &*room {
            let is_build = build_entity.as_ref().is_some_and(|b| b.0 == entity);
            if !is_build
                && !map_state.room(*x, *y, MapLayer::Main)
                && !matches!(room_state.action, ActionState::Destruct(_))
            {
                room_state.action =
                    ActionState::Destruct(time.elapsed_secs() + rand::random::<f32>() * 0.5);
            }
        }
        match room_state.action {
            ActionState::Construct(created) if elapsed - created >= 3.0 => {
                room_state.action = ActionState::Idle
            }
            ActionState::Destruct(created) if elapsed - created >= 3.0 => {
                commands.entity(entity).despawn_recursive();
            }
            _ => {}
        }
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

#[derive(Debug, Clone, ShaderType, Reflect)]
struct BuildMaterialSettings {
    created: f32,
    color: LinearRgba,
    direction: f32,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct BuildMaterial {
    #[uniform(100)]
    settings: BuildMaterialSettings,
}

pub type ExtendedBuildMaterial = ExtendedMaterial<StandardMaterial, BuildMaterial>;

impl MaterialExtension for BuildMaterial {
    fn fragment_shader() -> ShaderRef {
        "build.wgsl".into()
    }
}
