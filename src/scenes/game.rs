use bevy::prelude::*;
use build_material::BuildMaterialPlugin;
use builder::{BuilderPlugin, Enabled};
use camera::GameCameraPlugin;
use cargo::CargoPlugin;
use crusher::CrusherPlugin;
use enrichment::EnrichmentPlugin;
use furnace::FurnacePlugin;
use game_cursor::{GameCursorActive, GameCursorPlugin};
use generator::GeneratorPlugin;
use hook::{Hook, HookPlugin};
use light_consts::lux::CLEAR_SUNRISE;
use map_state::{MapStatePlugin, Structure};
use player::{PlayerPlugin, PlayerState};
use primary_block::{PrimaryBlock, PrimaryBlockPlugin};
use rock::RockPlugin;
use room::RoomPlugin;
use ui::{
    cargo_count::GameUiCargoCount,
    container::GameUiContainer,
    container_item::GameUiContainerItem,
    header::GameUiHeader,
    palette::{COLOR_CONTAINER, COLOR_HEADER, COLOR_TEXT},
    power_bar::GameUiPowerBar,
    GameUiPlugin,
};

use crate::components::clicked_event::{Clicked, Dehovered, Hovered};

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
mod ui;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameUiPlugin)
            .add_plugins(MapStatePlugin)
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
            .add_systems(Update, update.run_if(resource_exists::<GameEntities>))
            .insert_state(GameState::Idle)
            .insert_resource(TooltipState::default());
    }
}

#[derive(Resource)]
struct GameEntities {
    game_field: Entity,
    power_bar: Entity,
    tooltip: Entity,
    tooltip_title: Entity,
    tooltip_cost: Entity,
    tooltip_desc: Entity,
}

#[derive(Resource, Default)]
struct TooltipState {
    visible: bool,
    title: String,
    cost: String,
    desc: String,
}

fn setup(mut commands: Commands, root_entity: Res<AppSceneRoot>) {
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
        root.spawn(PrimaryBlock { x: 0, y: 0 })
            .with_child((Hook(false), Enabled));
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
    let mut power_bar = Entity::PLACEHOLDER;

    let construct_onclick = |structure: Structure| {
        move |_: Trigger<Clicked>, mut next_state: ResMut<NextState<PlayerState>>| {
            next_state.set(PlayerState::Construct(structure.clone()));
        }
    };

    let show_onhover = |title: &'static str, cost: &'static str, desc: &'static str| {
        |_: Trigger<Hovered>, mut tooltip: ResMut<TooltipState>| {
            tooltip.visible = true;
            tooltip.title = title.to_string();
            tooltip.cost = cost.to_string();
            tooltip.desc = desc.to_string();
        }
    };

    let hide_ondehover = |_: Trigger<Dehovered>, mut tooltip: ResMut<TooltipState>| {
        tooltip.visible = false;
    };

    let mut spawn_tooltip = |parent: &mut ChildBuilder| {
        tooltip = parent
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    width: Val::Px(200.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..Default::default()
                },
                BoxShadow::default(),
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
                parent
                    .spawn(GameUiContainer)
                    .with_child(
                        GameUiContainerItem::new("Silicon")
                            .footer(GameUiCargoCount::new(9999, 99999)),
                    )
                    .with_child(
                        GameUiContainerItem::new("Copper")
                            .footer(GameUiCargoCount::new(9999, 99999)),
                    )
                    .with_child(
                        GameUiContainerItem::new("Uranium")
                            .footer(GameUiCargoCount::new(9999, 99999)),
                    )
                    .with_child(
                        GameUiContainerItem::new("Ice").footer(GameUiCargoCount::new(9999, 99999)),
                    )
                    .with_child(
                        GameUiContainerItem::new("Aurelium")
                            .footer(GameUiCargoCount::new(9999, 99999)),
                    );
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
                    .id();
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(270.0),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    BackgroundColor(COLOR_CONTAINER),
                ));
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
                    .with_children(|parent| {
                        parent
                            .spawn(GameUiContainerItem::new("Room").button().image("room.png"))
                            .observe(construct_onclick(Structure::EmptyRoom))
                            .observe(show_onhover("Room", "???", "Useful for building a base"))
                            .observe(hide_ondehover);
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(
                                GameUiContainerItem::new("Furnace")
                                    .button()
                                    .image("furnace.png"),
                            )
                            .observe(construct_onclick(Structure::Furnace))
                            .observe(show_onhover("Furnace", "???", "Melts ores and ice"))
                            .observe(hide_ondehover);
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(
                                GameUiContainerItem::new("Cargo")
                                    .button()
                                    .image("cargo.png"),
                            )
                            .observe(construct_onclick(Structure::Cargo))
                            .observe(show_onhover(
                                "Cargo",
                                "???",
                                "Extends your storing capabilities!",
                            ))
                            .observe(hide_ondehover);
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(
                                GameUiContainerItem::new("Crusher")
                                    .button()
                                    .image("crusher.png"),
                            )
                            .observe(construct_onclick(Structure::Crusher))
                            .observe(show_onhover(
                                "Crusher",
                                "???",
                                "Crushes stones into the silicon dust",
                            ))
                            .observe(hide_ondehover);
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(
                                GameUiContainerItem::new("Generator")
                                    .button()
                                    .image("generator.png"),
                            )
                            .observe(construct_onclick(Structure::Generator))
                            .observe(show_onhover("Generator", "???", "Generates power"))
                            .observe(hide_ondehover);
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(GameUiContainerItem::new("Hook").button().image("hook.png"))
                            .observe(construct_onclick(Structure::Hook))
                            .observe(show_onhover("Hook", "???", "Automatic hook"))
                            .observe(hide_ondehover);
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(
                                GameUiContainerItem::new("Enrichment station")
                                    .button()
                                    .image("enrichment.png"),
                            )
                            .observe(construct_onclick(Structure::Enrichment))
                            .observe(show_onhover("Enrichment station", "???", "Makes batteries"))
                            .observe(hide_ondehover);
                    });
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
        tooltip,
        tooltip_title,
        tooltip_cost,
        tooltip_desc,
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
) {
    if let Ok(mut power_bar) = power_bars.get_mut(state.power_bar) {
        power_bar.power = (power_bar.power + time.delta_secs()).fract();
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
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    Idle,
    Pause,
}
