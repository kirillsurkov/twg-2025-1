use bevy::{pbr::ExtendedMaterial, prelude::*};

use crate::{
    components::{
        material_modifier::MaterialModifier, procedural_material::ExtendedProceduralMaterial,
    },
    scenes::AppSceneRoot,
};

use super::{
    build_material::{BuildMaterial, BuildMaterialSettings, ExtendedBuildMaterial},
    cargo::Cargo,
    crusher::Crusher,
    enrichment::Enrichment,
    furnace::Furnace,
    game_cursor::{CursorLayer, GameCursor},
    generator::Generator,
    hook::Hook,
    map_state::{MapLayer, MapNode, MapState},
    player::PlayerState,
    primary_block::PrimaryBlock,
    room::Room,
};

pub struct BuilderPlugin;

impl Plugin for BuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            (update_material, transit_state, construct, update, destruct).chain(),
        );
    }
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
pub struct NodeState {
    pub highlight: HighlightState,
    pub action: ActionState,
}

#[derive(Component)]
pub struct Ready;

#[derive(Component)]
pub struct Enabled;

#[derive(Resource)]
pub struct BuildEntity(pub Entity);

fn transit_state(
    mut commands: Commands,
    mut transition_events: EventReader<StateTransitionEvent<PlayerState>>,
    build_entity: Option<Res<BuildEntity>>,
) {
    for event in transition_events.read() {
        match event.exited {
            Some(PlayerState::Construct(_)) => {
                if let Some(BuildEntity(entity)) = build_entity.as_deref() {
                    commands.entity(*entity).despawn_recursive();
                    commands.remove_resource::<BuildEntity>();
                }
            }
            _ => {}
        }
    }
}

fn construct(
    mut commands: Commands,
    mut nodes: Query<(&mut NodeState, &mut Transform)>,
    mut map_state: ResMut<MapState>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    player_state: Res<State<PlayerState>>,
    root_entity: Res<AppSceneRoot>,
    game_cursor: Option<Res<GameCursor>>,
    build_entity: Option<Res<BuildEntity>>,
    time: Res<Time>,
) {
    let Some(game_cursor) = game_cursor.as_ref() else {
        if let Some(BuildEntity(entity)) = build_entity.as_deref() {
            commands.entity(*entity).despawn_recursive();
            commands.remove_resource::<BuildEntity>();
        }
        return;
    };

    let PlayerState::Construct(node) = player_state.get() else {
        return;
    };

    let Some(BuildEntity(entity)) = build_entity.as_deref() else {
        let entity = match node {
            MapNode::PrimaryBlock => commands.spawn(PrimaryBlock),
            MapNode::EmptyRoom => commands.spawn(Room),
            MapNode::Furnace => commands.spawn(Furnace),
            MapNode::Generator => commands.spawn(Generator),
            MapNode::Crusher => commands.spawn(Crusher),
            MapNode::Cargo => commands.spawn(Cargo),
            MapNode::Hook => commands.spawn(Hook(true)),
            MapNode::Enrichment => commands.spawn(Enrichment),
        }
        .insert(NodeState {
            action: ActionState::Idle,
            highlight: HighlightState::Green,
        })
        .insert(Transform::from_xyz(game_cursor.fx, game_cursor.fy, 0.0))
        .id();
        commands.entity(root_entity.world).add_child(entity);
        commands.insert_resource(BuildEntity(entity));
        return;
    };

    let Ok((mut node_state, mut transform)) = nodes.get_mut(*entity) else {
        return;
    };

    let available = map_state.is_available(game_cursor.x, game_cursor.y, node.clone());

    if available {
        transform.translation =
            GameCursor::game_to_world(game_cursor.x, game_cursor.y, CursorLayer::Room).extend(0.0);
        if node_state.highlight != HighlightState::Green {
            node_state.highlight = HighlightState::Green;
        }
    } else {
        transform.translation.x = game_cursor.fx;
        transform.translation.y = game_cursor.fy;
        transform.translation.z = 0.1;
        if node_state.highlight != HighlightState::Red {
            node_state.highlight = HighlightState::Red;
        }
    }

    if available && game_cursor.just_pressed {
        node_state.action = ActionState::Construct(time.elapsed_secs());
        node_state.highlight = HighlightState::None;
        next_player_state.set(PlayerState::Idle);
        commands.remove_resource::<BuildEntity>();
        map_state.add_room(game_cursor.x, game_cursor.y, node.clone());
        for (cargo, count) in node.recipe() {
            map_state.harvest(cargo, -count);
        }
    }
}

fn destruct(
    mut nodes: Query<(Entity, &mut NodeState, &Transform)>,
    mut map_state: ResMut<MapState>,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    player_state: Res<State<PlayerState>>,
    game_cursor: Option<Res<GameCursor>>,
    build_entity: Option<Res<BuildEntity>>,
) {
    let PlayerState::Destruct = player_state.get() else {
        return;
    };

    for (entity, mut node_state, transform) in nodes.iter_mut() {
        if build_entity.as_ref().is_some_and(|e| e.0 == entity) {
            continue;
        }

        let Vec2 { x, y } = transform.translation.xy();
        let IVec2 { x, y } = GameCursor::world_to_game(x, y, CursorLayer::Room);

        if map_state.is_node(x, y, MapLayer::Build) {
            node_state.highlight = HighlightState::None;
            continue;
        }

        let is_selected = match game_cursor.as_ref() {
            Some(game_cursor) => {
                if let PlayerState::Destruct = player_state.get() {
                    if game_cursor.just_pressed {
                        map_state.remove_room(x, y, MapLayer::Main);
                        next_player_state.set(PlayerState::Idle);
                    }
                }
                x == game_cursor.x && y == game_cursor.y
            }
            None => false,
        };

        if is_selected && node_state.highlight != HighlightState::Red {
            node_state.highlight = HighlightState::Red;
        }
        if !is_selected && node_state.highlight != HighlightState::Orange {
            node_state.highlight = HighlightState::Orange;
        }
    }

    map_state.sync_build();
    if let Some(game_cursor) = game_cursor.as_ref() {
        if let PlayerState::Destruct = player_state.get() {
            map_state.remove_room(game_cursor.x, game_cursor.y, MapLayer::Build);
        };
    }
}

fn update_material(mut commands: Commands, nodes: Query<(Entity, &NodeState), With<Ready>>) {
    for (entity, node_state) in nodes.iter() {
        let mut entity = commands.entity(entity);

        entity
            .remove::<MaterialModifier<StandardMaterial>>()
            .remove::<MaterialModifier<ExtendedBuildMaterial>>()
            .remove::<MaterialModifier<ExtendedProceduralMaterial>>();

        let highlight_white = LinearRgba::WHITE * 0.01;
        let highlight_green = LinearRgba::new(0.0, 1.0, 0.0, 1.0);
        let highlight_orange = LinearRgba::new(1.0, 0.5, 0.0, 1.0);
        let highlight_red = LinearRgba::new(1.0, 0.0, 0.0, 1.0);

        let with_highlight = |mut mat: StandardMaterial, highlight| {
            mat.base_color.set_alpha(0.01);
            mat.emissive = highlight * 10000.0;
            mat.alpha_mode = AlphaMode::Blend;
            mat
        };

        let procedural_inactive =
            MaterialModifier::new(move |mut mat: ExtendedProceduralMaterial| {
                mat.base.base_color.set_alpha(0.5);
                mat.base.emissive = LinearRgba::NONE;
                mat
            });

        match (node_state.action, node_state.highlight) {
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
                entity
                    .insert(procedural_inactive)
                    .insert(MaterialModifier::new(move |mat: StandardMaterial| {
                        with_highlight(mat, highlight_green)
                    }));
            }
            (ActionState::Idle, HighlightState::Orange) => {
                entity
                    .insert(procedural_inactive)
                    .insert(MaterialModifier::new(move |mat: StandardMaterial| {
                        with_highlight(mat, highlight_orange)
                    }));
            }
            (ActionState::Idle, HighlightState::Red) => {
                entity
                    .insert(procedural_inactive)
                    .insert(MaterialModifier::new(move |mat: StandardMaterial| {
                        with_highlight(mat, highlight_red)
                    }));
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
                    .insert(procedural_inactive);
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

fn update(
    mut commands: Commands,
    mut query: Query<(Entity, &mut NodeState, &Transform)>,
    build_entity: Option<Res<BuildEntity>>,
    game_cursor: Option<Res<GameCursor>>,
    map_state: Res<MapState>,
    player_state: Res<State<PlayerState>>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();

    for (entity, mut state, transform) in query.iter_mut() {
        let enabled = state.action == ActionState::Idle
            && !build_entity.as_ref().is_some_and(|e| e.0 == entity);

        if enabled {
            commands.entity(entity).insert(Enabled);
        } else {
            commands.entity(entity).remove::<Enabled>();
        }

        match state.action {
            ActionState::Construct(created) if elapsed - created >= 3.0 => {
                state.action = ActionState::Idle
            }
            ActionState::Destruct(created) if elapsed - created >= 3.0 => {
                commands.entity(entity).despawn_recursive();
            }
            ActionState::Idle => {
                let Vec2 { x, y } = transform.translation.xy();
                let IVec2 { x, y } = GameCursor::world_to_game(x, y, CursorLayer::Room);

                let is_selected = match (game_cursor.as_ref(), player_state.get()) {
                    (_, PlayerState::Interact(ix, iy)) => x == *ix && y == *iy,
                    (Some(game_cursor), _) => x == game_cursor.x && y == game_cursor.y,
                    _ => false,
                };

                if !build_entity.as_ref().is_some_and(|b| b.0 == entity)
                    && !map_state.is_node(x, y, MapLayer::Main)
                    && !matches!(state.action, ActionState::Destruct(_))
                {
                    state.action = ActionState::Destruct(time.elapsed_secs());
                }

                if enabled {
                    state.highlight = if is_selected {
                        HighlightState::White
                    } else {
                        HighlightState::None
                    };
                }
            }
            _ => {}
        }
    }
}
