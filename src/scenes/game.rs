use bevy::prelude::*;
use build_material::BuildMaterialPlugin;
use builder::{BuilderPlugin, Enabled};
use camera::GameCameraPlugin;
use furnace::FurnacePlugin;
use game_cursor::{GameCursorActive, GameCursorPlugin};
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
    palette::{COLOR_CONTAINER, COLOR_HEADER},
    power_bar::GameUiPowerBar,
    GameUiPlugin,
};

use crate::components::clicked_event::Clicked;

use super::{AppSceneRoot, AppState};

mod build_material;
mod builder;
mod camera;
mod game_cursor;
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
            .add_plugins(RockPlugin)
            .add_plugins(HookPlugin)
            .add_systems(OnEnter(AppState::Game), setup)
            .add_systems(Update, update.run_if(resource_exists::<GameEntities>))
            .insert_state(GameState::Idle);
    }
}

#[derive(Resource)]
struct GameEntities {
    game_field: Entity,
    power_bar: Entity,
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

    let mut game_field = Entity::PLACEHOLDER;
    let mut power_bar = Entity::PLACEHOLDER;

    commands.entity(root_entity.ui).with_children(|root| {
        root.spawn(Node {
            width: Val::Px(380.0),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip(),
            ..Default::default()
        })
        .insert((
            BoxShadow {
                blur_radius: Val::Px(5.0),
                spread_radius: Val::Px(5.0),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                color: Color::BLACK,
            },
            ZIndex(i32::MAX),
        ))
        .with_children(|parent| {
            parent.spawn(GameUiHeader::new("Cargo"));
            parent
                .spawn(GameUiContainer)
                .with_child(
                    GameUiContainerItem::new("Silicon").footer(GameUiCargoCount::new(9999, 99999)),
                )
                .with_child(
                    GameUiContainerItem::new("Copper").footer(GameUiCargoCount::new(9999, 99999)),
                )
                .with_child(
                    GameUiContainerItem::new("Uranium").footer(GameUiCargoCount::new(9999, 99999)),
                )
                .with_child(
                    GameUiContainerItem::new("Ice").footer(GameUiCargoCount::new(9999, 99999)),
                )
                .with_child(
                    GameUiContainerItem::new("Aurelium").footer(GameUiCargoCount::new(9999, 99999)),
                );
        });

        root.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
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

        root.spawn(Node {
            width: Val::Px(320.0),
            height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip(),
            ..Default::default()
        })
        .insert((
            BoxShadow {
                blur_radius: Val::Px(5.0),
                spread_radius: Val::Px(5.0),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                color: Color::BLACK,
            },
            ZIndex(i32::MAX),
        ))
        .with_children(|parent| {
            parent.spawn(GameUiHeader::new("Build"));
            parent
                .spawn(GameUiContainer)
                .with_children(|parent| {
                    parent
                        .spawn(GameUiContainerItem::new("Room").button())
                        .observe(
                        |_: Trigger<Clicked>, mut next_state: ResMut<NextState<PlayerState>>| {
                            next_state.set(PlayerState::Construct(Structure::EmptyRoom));
                        },
                    );
                })
                .with_children(|parent| {
                    parent
                        .spawn(GameUiContainerItem::new("Furnace").button())
                        .observe(|_: Trigger<Clicked>| {
                            println!("Furnace clicked");
                        });
                })
                .with_children(|parent| {
                    parent
                        .spawn(GameUiContainerItem::new("Cargo").button())
                        .observe(|_: Trigger<Clicked>| {
                            println!("Cargo clicked");
                        });
                })
                .with_children(|parent| {
                    parent
                        .spawn(GameUiContainerItem::new("Generator").button())
                        .observe(|_: Trigger<Clicked>| {
                            println!("Generator clicked");
                        });
                })
                .with_children(|parent| {
                    parent
                        .spawn(GameUiContainerItem::new("Hook").button())
                        .observe(
                        |_: Trigger<Clicked>, mut next_state: ResMut<NextState<PlayerState>>| {
                            next_state.set(PlayerState::Construct(Structure::Hook));
                        },
                    );
                });
        });
    });

    commands.insert_resource(GameEntities {
        power_bar,
        game_field,
    });
}

fn update(
    mut commands: Commands,
    state: Res<GameEntities>,
    mut power_bars: Query<&mut GameUiPowerBar>,
    time: Res<Time>,
    interactions: Query<&Interaction>,
) {
    if let Ok(mut power_bar) = power_bars.get_mut(state.power_bar) {
        power_bar.power = (power_bar.power + time.delta_secs()).fract();
    }

    match interactions.get(state.game_field) {
        Err(_) | Ok(Interaction::None) => commands.remove_resource::<GameCursorActive>(),
        _ => commands.insert_resource(GameCursorActive),
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    Idle,
    Pause,
}
