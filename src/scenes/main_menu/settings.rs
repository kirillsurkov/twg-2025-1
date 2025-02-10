use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::DebandDither},
    prelude::*,
};

use crate::{
    components::{background::RenderBackground, game_button::GameButton, mouse_event::Clicked},
    scenes::game::ui::palette::{COLOR_HIGHLIGHT_DARK, COLOR_POWER_HIGH, COLOR_POWER_LOW},
};

use super::{AppSceneRoot, AppState};

pub struct MainMenuSettingsPlugin;

impl Plugin for MainMenuSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenuSettings), setup)
            .add_systems(Update, update.run_if(in_state(AppState::MainMenuSettings)))
            .insert_resource(GameSettings {
                music_volume: 0.2,
                effects_volume: 0.2,
                oit_layers: 4,
                bloom: true,
                fullscreen: false,
            });
    }
}

#[derive(Resource)]
pub struct GameSettings {
    pub music_volume: f32,
    pub effects_volume: f32,
    pub oit_layers: u32,
    pub bloom: bool,
    pub fullscreen: bool,
}

#[derive(Resource)]
struct Entities {
    music_bar: Entity,
    effects_bar: Entity,
    oit_count: Entity,
    bloom_indicator: Entity,
    fullscreen_indicator: Entity,
}

fn setup(mut commands: Commands, root_entity: Res<AppSceneRoot>) {
    commands.entity(root_entity.world).with_child((
        Camera3d::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::None,
            ..Default::default()
        },
        RenderBackground,
        Msaa::Off,
        Bloom::NATURAL,
        DebandDither::Enabled,
    ));
    let spacer = Node {
        height: Val::Px(5.0),
        ..Default::default()
    };
    let apply = commands
        .spawn(GameButton::new("Apply", 200.0))
        .observe(
            |_: Trigger<Clicked>, mut next: ResMut<NextState<AppState>>| {
                next.set(AppState::MainMenu)
            },
        )
        .id();

    let music_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    let effects_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    let oit_count = commands.spawn(Text::default()).id();
    let bloom_indicator = commands.spawn(Text::default()).id();
    let fullscreen_indicator = commands.spawn(Text::default()).id();

    commands.insert_resource(Entities {
        music_bar,
        effects_bar,
        oit_count,
        bloom_indicator,
        fullscreen_indicator,
    });

    commands.entity(root_entity.ui).with_children(|root| {
        root.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_child((
                    Node {
                        width: Val::Px(100.0),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    Text::new("Music volume"),
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                .with_children(|parent| {
                    parent.spawn(GameButton::new("-", 0.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.music_volume = (settings.music_volume - 0.1).max(0.0);
                        },
                    );
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                ..Default::default()
                            },
                            Outline {
                                color: COLOR_HIGHLIGHT_DARK,
                                offset: Val::ZERO,
                                width: Val::Px(3.0),
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .add_child(music_bar);
                })
                .with_children(|parent| {
                    parent.spawn(GameButton::new("+", 120.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.music_volume = (settings.music_volume + 0.1).min(1.0);
                        },
                    );
                });
            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    display: Display::None,
                    ..Default::default()
                })
                .with_child((
                    Node {
                        width: Val::Px(100.0),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    Text::new("Effects volume"),
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                .with_children(|parent| {
                    parent.spawn(GameButton::new("-", 0.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.effects_volume = (settings.effects_volume - 0.1).max(0.0);
                        },
                    );
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                ..Default::default()
                            },
                            Outline {
                                color: COLOR_HIGHLIGHT_DARK,
                                offset: Val::ZERO,
                                width: Val::Px(3.0),
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .add_child(effects_bar);
                })
                .with_children(|parent| {
                    parent.spawn(GameButton::new("+", 120.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.effects_volume = (settings.effects_volume + 0.1).min(1.0);
                        },
                    );
                });
            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_child((
                    Node {
                        width: Val::Px(100.0),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    Text::new("OIT layers"),
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                .with_children(|parent| {
                    parent.spawn(GameButton::new("-", 0.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.oit_layers = (settings.oit_layers - 1).max(1);
                        },
                    );
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            Outline {
                                color: COLOR_HIGHLIGHT_DARK,
                                offset: Val::ZERO,
                                width: Val::Px(3.0),
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .add_child(oit_count);
                })
                .with_children(|parent| {
                    parent.spawn(GameButton::new("+", 120.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.oit_layers = (settings.oit_layers + 1).min(16);
                        },
                    );
                });
            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_child((
                    Node {
                        width: Val::Px(100.0),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    Text::new("Bloom"),
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                .with_children(|parent| {
                    parent.spawn(GameButton::new("-", 0.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.bloom = false;
                        },
                    );
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            Outline {
                                color: COLOR_HIGHLIGHT_DARK,
                                offset: Val::ZERO,
                                width: Val::Px(3.0),
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .add_child(bloom_indicator);
                })
                .with_children(|parent| {
                    parent.spawn(GameButton::new("+", 120.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.bloom = true;
                        },
                    );
                });

            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_child((
                    Node {
                        width: Val::Px(100.0),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    Text::new("Fullscreen"),
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                .with_children(|parent| {
                    parent.spawn(GameButton::new("-", 0.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.fullscreen = false;
                        },
                    );
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            },
                            Outline {
                                color: COLOR_HIGHLIGHT_DARK,
                                offset: Val::ZERO,
                                width: Val::Px(3.0),
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .add_child(fullscreen_indicator);
                })
                .with_children(|parent| {
                    parent.spawn(GameButton::new("+", 120.0)).observe(
                        |_: Trigger<Clicked>, mut settings: ResMut<GameSettings>| {
                            settings.fullscreen = true;
                        },
                    );
                });

            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::End,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_child(spacer.clone())
                .add_children(&[apply]);
        });
    });
}

fn update(
    settings: Res<GameSettings>,
    entities: Option<Res<Entities>>,
    mut backgrounds: Query<(&mut Node, &mut BackgroundColor)>,
    mut texts: Query<&mut Text>,
) {
    let Some(entities) = entities.as_ref() else {
        return;
    };

    if let Ok((mut node, mut color)) = backgrounds.get_mut(entities.music_bar) {
        node.width = Val::Percent(settings.music_volume * 100.0);
        color.0 = COLOR_POWER_LOW.mix(&COLOR_POWER_HIGH, settings.music_volume);
    }

    if let Ok((mut node, mut color)) = backgrounds.get_mut(entities.effects_bar) {
        node.width = Val::Percent(settings.effects_volume * 100.0);
        color.0 = COLOR_POWER_LOW.mix(&COLOR_POWER_HIGH, settings.effects_volume);
    }

    if let Ok(mut text) = texts.get_mut(entities.oit_count) {
        text.0 = format!("{}", settings.oit_layers);
    }

    if let Ok(mut text) = texts.get_mut(entities.bloom_indicator) {
        text.0 = if settings.bloom { "ON" } else { "OFF" }.to_string();
    }

    if let Ok(mut text) = texts.get_mut(entities.fullscreen_indicator) {
        text.0 = if settings.fullscreen { "ON" } else { "OFF" }.to_string();
    }
}
