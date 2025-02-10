use bevy::prelude::*;
use build_material::BuildMaterialPlugin;
use builder::{ActionState, BuilderPlugin, Enabled, HighlightState, NodeState};
use camera::GameCameraPlugin;
use cargo::CargoPlugin;
use crusher::CrusherPlugin;
use enrichment::EnrichmentPlugin;
use furnace::FurnacePlugin;
use game_cursor::{GameCursor, GameCursorActive, GameCursorPlugin};
use generator::GeneratorPlugin;
use hook::{Hook, HookPlugin};
use light_consts::lux::CLEAR_SUNRISE;
use map_state::{Cargo, MapLayer, MapNode, MapState, MapStatePlugin};
use player::{PlayerPlugin, PlayerState};
use primary_block::{PrimaryBlock, PrimaryBlockPlugin};
use rock::RockPlugin;
use room::RoomPlugin;
use strum::IntoEnumIterator;
use ui::{
    cargo_count::GameUiCargoCount,
    container::GameUiContainer,
    container_item::GameUiContainerItem,
    header::GameUiHeader,
    palette::{COLOR_CONTAINER, COLOR_HEADER, COLOR_HIGHLIGHT_DARK, COLOR_TEXT},
    power_bar::GameUiPowerBar,
    GameUiPlugin,
};

use crate::components::{
    game_button::GameButton,
    mouse_event::{Clicked, Dehovered, Hovered},
    music_player::MusicPlayerPlugin,
};

use super::{AppSceneRoot, AppState};

mod build_material;
mod builder;
mod camera;
mod cargo;
mod crusher;
mod enrichment;
mod furnace;
mod game_cursor;
mod generator;
mod hook;
mod map_state;
mod player;
mod primary_block;
mod rock;
mod room;
pub mod ui;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameUiPlugin)
            .add_plugins(MapStatePlugin)
            .add_plugins(MusicPlayerPlugin)
            .add_plugins(GameCameraPlugin(Vec3::new(0.0, 0.0, 15.0)))
            .add_plugins(PlayerPlugin)
            .add_plugins(GameCursorPlugin)
            .add_plugins(BuildMaterialPlugin)
            .add_plugins(BuilderPlugin)
            .add_plugins(PrimaryBlockPlugin)
            .add_plugins(RoomPlugin)
            .add_plugins(CargoPlugin)
            .add_plugins(FurnacePlugin)
            .add_plugins(GeneratorPlugin)
            .add_plugins(EnrichmentPlugin)
            .add_plugins(CrusherPlugin)
            .add_plugins(RockPlugin)
            .add_plugins(HookPlugin)
            .add_systems(OnEnter(AppState::Game), setup)
            .add_systems(
                Update,
                (
                    update.run_if(resource_exists::<GameEntities>),
                    process_spawn_requests,
                )
                    .chain(),
            )
            .insert_state(GameState::Idle)
            .insert_resource(TooltipState::default());
    }
}

#[derive(Resource)]
struct GameEntities {
    game_field: Entity,
    pause_menu: Entity,
    power_bar: Entity,
    tooltip: Entity,
    tooltip_title: Entity,
    tooltip_cost: Entity,
    tooltip_desc: Entity,
    info_thumbnail: Entity,
    cargo_counts: Vec<(Cargo, Entity)>,
}

#[derive(Resource, Default)]
struct TooltipState {
    visible: bool,
    title: String,
    cost: String,
    desc: String,
}

#[derive(Resource)]
struct SpawnRequest(MapNode);

fn process_spawn_requests(
    mut commands: Commands,
    mut next_state: ResMut<NextState<PlayerState>>,
    map_state: Res<MapState>,
    request: Option<Res<SpawnRequest>>,
) {
    let Some(SpawnRequest(node)) = request.as_deref() else {
        return;
    };

    commands.remove_resource::<SpawnRequest>();

    let mut success = true;
    for (cargo, count) in node.recipe() {
        if map_state.cargo_count(cargo.clone()).0 < count {
            success = false;
            break;
        }
    }

    if success {
        next_state.set(PlayerState::Construct(node.clone()));
    } else {
    }
}

fn item_spawner(node: MapNode) -> impl Fn(&mut ChildBuilder) {
    move |parent| {
        parent
            .spawn(
                GameUiContainerItem::new(node.name())
                    .button()
                    .image(node.thumbnail()),
            )
            .observe({
                let node = node.clone();
                move |_: Trigger<Clicked>, mut commands: Commands| {
                    commands.insert_resource(SpawnRequest(node.clone()));
                }
            })
            .observe({
                let node = node.clone();
                move |_: Trigger<Hovered>, mut tooltip: ResMut<TooltipState>| {
                    tooltip.visible = true;
                    tooltip.title = node.name().to_string();
                    tooltip.cost = node
                        .recipe()
                        .into_iter()
                        .map(|(cargo, cnt)| format!("{}: {cnt}", cargo.name()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    tooltip.desc = node.desc().to_string();
                }
            })
            .observe(|_: Trigger<Dehovered>, mut tooltip: ResMut<TooltipState>| {
                tooltip.visible = false;
            });
    }
}

fn setup(
    mut commands: Commands,
    mut map_state: ResMut<MapState>,
    root_entity: Res<AppSceneRoot>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    next_state.set(GameState::Idle);

    let directional_light = |x, y| {
        (
            DirectionalLight {
                illuminance: CLEAR_SUNRISE,
                shadows_enabled: true,
                ..Default::default()
            },
            Transform::from_xyz(x, y, 10.0).looking_at(Vec3::ZERO, Vec3::Z),
        )
    };

    commands.entity(root_entity.world).with_children(|root| {
        root.spawn(directional_light(1.0, 1.0));
        root.spawn(directional_light(1.0, -1.0));
        root.spawn(directional_light(-1.0, 1.0));
        root.spawn(directional_light(-1.0, -1.0));
        root.spawn((
            PrimaryBlock,
            NodeState {
                action: ActionState::Idle,
                highlight: HighlightState::None,
            },
        ))
        .with_child((Hook(false), Enabled));
        map_state.add_primary_block(0, 0);
    });

    // commands.spawn((
    //     AudioPlayer::new(asset_server.load("Cojam - Milky Main Menu.ogg")),
    //     PlaybackSettings {
    //         mode: PlaybackMode::Loop,
    //         speed: 0.5,
    //         ..Default::default()
    //     },
    // ));

    let mut tooltip = Entity::PLACEHOLDER;
    let mut tooltip_title = Entity::PLACEHOLDER;
    let mut tooltip_cost = Entity::PLACEHOLDER;
    let mut tooltip_desc = Entity::PLACEHOLDER;
    let mut game_field = Entity::PLACEHOLDER;
    let mut pause_menu = Entity::PLACEHOLDER;
    let mut power_bar = Entity::PLACEHOLDER;
    let mut info_thumbnail = Entity::PLACEHOLDER;
    let mut cargo_counts = vec![];

    let mut spawn_tooltip = |parent: &mut ChildBuilder| {
        tooltip = parent
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    width: Val::Px(300.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..Default::default()
                },
                BoxShadow::default(),
                BorderRadius::all(Val::Px(5.0)),
                Outline::new(Val::Px(3.0), Val::ZERO, COLOR_HIGHLIGHT_DARK),
                BackgroundColor(COLOR_CONTAINER),
                ZIndex(i32::MAX),
            ))
            .with_children(|parent| {
                let spacer = (
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(3.0),
                        padding: UiRect::bottom(Val::Px(5.0)),
                        ..Default::default()
                    },
                    BackgroundColor(COLOR_TEXT),
                );
                tooltip_title = parent.spawn(Text::default()).id();
                parent.spawn(spacer.clone());
                tooltip_cost = parent.spawn(Text::default()).id();
                parent.spawn(spacer.clone());
                tooltip_desc = parent.spawn(Text::default()).id();
            })
            .id();
    };

    let mut spawn_cargo_counts = |parent: &mut ChildBuilder| {
        parent
            .spawn(Node {
                width: Val::Px(380.0),
                height: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..Default::default()
            })
            .insert((BoxShadow {
                blur_radius: Val::Px(5.0),
                spread_radius: Val::Px(5.0),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                color: Color::BLACK,
            },))
            .with_children(|parent| {
                parent.spawn(GameUiHeader::new("Cargo"));
                parent.spawn(GameUiContainer).with_children(|parent| {
                    for cargo in Cargo::iter() {
                        let footer = parent.spawn(GameUiCargoCount::new(0.0, 0.0)).id();
                        cargo_counts.push((cargo.clone(), footer));
                        parent.spawn(GameUiContainerItem::new(cargo.name()).footer(footer));
                    }
                });
            });
    };

    let mut spawn_center = |parent: &mut ChildBuilder| {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            })
            .insert(ZIndex(i32::MIN))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(80.0),
                            padding: UiRect::horizontal(Val::Px(25.0))
                                .with_top(Val::Px(15.0))
                                .with_bottom(Val::Px(15.0)),
                            flex_shrink: 0.0,
                            ..Default::default()
                        },
                        BackgroundColor(COLOR_HEADER),
                    ))
                    .with_children(|parent| {
                        power_bar = parent.spawn(GameUiPowerBar::new()).id();
                    });
                game_field = parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Default::default()
                    })
                    .insert(Button)
                    .with_children(|parent| {
                        pause_menu = parent
                            .spawn((
                                Visibility::Hidden,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    ..Default::default()
                                },
                                BackgroundColor(Color::BLACK.with_alpha(0.8)),
                            ))
                            .with_child(Text::new("PAUSED"))
                            .with_child(Node {
                                height: Val::Px(100.0),
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn(GameButton::new("Resume", 120.0)).observe(
                                    |_: Trigger<Clicked>, mut next: ResMut<NextState<GameState>>| {
                                        next.set(GameState::Idle);
                                    },
                                );
                            })
                            .with_children(|parent| {
                                parent.spawn(GameButton::new("Exit", 0.0)).observe(
                                    |_: Trigger<Clicked>, mut next: ResMut<NextState<AppState>>| {
                                        next.set(AppState::MainMenu);
                                    },
                                );
                            })
                            .id();
                    })
                    .id();
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            display: Display::None,
                            ..Default::default()
                        },
                        BackgroundColor(COLOR_CONTAINER),
                    ))
                    .with_children(|parent| {
                        info_thumbnail = parent
                            .spawn(Node {
                                width: Val::Px(100.0),
                                height: Val::Px(100.0),
                                ..Default::default()
                            })
                            .id();
                    });
            });
    };

    let mut spawn_build_options = |parent: &mut ChildBuilder| {
        parent
            .spawn(Node {
                width: Val::Px(320.0),
                height: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..Default::default()
            })
            .insert((BoxShadow {
                blur_radius: Val::Px(5.0),
                spread_radius: Val::Px(5.0),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                color: Color::BLACK,
            },))
            .with_children(|parent| {
                parent.spawn(GameUiHeader::new("Build"));
                parent
                    .spawn(GameUiContainer)
                    .with_children(item_spawner(MapNode::EmptyRoom))
                    .with_children(item_spawner(MapNode::Furnace))
                    .with_children(item_spawner(MapNode::Cargo))
                    .with_children(item_spawner(MapNode::Crusher))
                    .with_children(item_spawner(MapNode::Generator))
                    .with_children(item_spawner(MapNode::Hook))
                    .with_children(item_spawner(MapNode::Enrichment));
            });
    };

    commands.entity(root_entity.ui).with_children(|root| {
        spawn_tooltip(root);
        spawn_cargo_counts(root);
        spawn_center(root);
        spawn_build_options(root);
    });

    commands.insert_resource(GameEntities {
        power_bar,
        game_field,
        pause_menu,
        tooltip,
        tooltip_title,
        tooltip_cost,
        tooltip_desc,
        info_thumbnail,
        cargo_counts,
    });
}

fn update(
    mut commands: Commands,
    state: Res<GameEntities>,
    mut power_bars: Query<&mut GameUiPowerBar>,
    time: Res<Time>,
    interactions: Query<&Interaction>,
    mut nodes: Query<&mut Node>,
    mut texts: Query<&mut Text>,
    mut visibilities: Query<&mut Visibility>,
    window: Query<&Window>,
    tooltip_state: Res<TooltipState>,
    map_state: Res<MapState>,
    player_state: Res<State<PlayerState>>,
    game_cursor: Option<Res<GameCursor>>,
    assets: Res<AssetServer>,
    mut cargo_counts: Query<&mut GameUiCargoCount>,
    game_state: Res<State<GameState>>,
) {
    if let Ok(mut visibility) = visibilities.get_mut(state.pause_menu) {
        *visibility = match game_state.get() {
            GameState::Idle => Visibility::Hidden,
            GameState::Pause => Visibility::Inherited,
        };
    }

    if let Ok(mut power_bar) = power_bars.get_mut(state.power_bar) {
        let dir = (map_state.energy_ratio() - power_bar.power)
            .max(-1.0)
            .min(1.0);

        power_bar.power += 10.0 * time.delta_secs() * dir;
    }

    match interactions.get(state.game_field) {
        Err(_) | Ok(Interaction::None) => commands.remove_resource::<GameCursorActive>(),
        _ => commands.insert_resource(GameCursorActive),
    }

    let Ok(window) = window.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    if let Ok(mut node) = nodes.get_mut(state.tooltip) {
        let Vec2 { x: w, y: h } = window.size();
        node.right = Val::Percent(100.0 * (w - cursor_pos.x as f32) / w);
        node.top = Val::Percent(100.0 * cursor_pos.y as f32 / h);
    }

    if let Ok(mut visibility) = visibilities.get_mut(state.tooltip) {
        *visibility = if tooltip_state.visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut text) = texts.get_mut(state.tooltip_title) {
        text.0 = tooltip_state.title.clone();
    }
    if let Ok(mut text) = texts.get_mut(state.tooltip_cost) {
        text.0 = tooltip_state.cost.clone();
    }
    if let Ok(mut text) = texts.get_mut(state.tooltip_desc) {
        text.0 = tooltip_state.desc.clone();
    }

    if let Some(mut thumbnail) = commands.get_entity(state.info_thumbnail) {
        if let Some((x, y)) = match (game_cursor.as_ref(), player_state.get()) {
            (_, PlayerState::Interact(ix, iy)) => Some((*ix, *iy)),
            (Some(game_cursor), PlayerState::Idle) => Some((game_cursor.x, game_cursor.y)),
            _ => None,
        } {
            if let Some(node) = map_state.node(x, y, MapLayer::Main) {
                thumbnail.insert(ImageNode::new(assets.load(node.thumbnail())));
            }
        }
    }

    for (cargo, count) in &state.cargo_counts {
        if let Ok(mut count) = cargo_counts.get_mut(*count) {
            let (cur, max) = map_state.cargo_count(cargo.clone());
            count.cur = cur;
            count.max = max;
        }
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    Idle,
    Pause,
}
