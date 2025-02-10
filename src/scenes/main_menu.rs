use bevy::{
    audio::PlaybackMode,
    core_pipeline::{bloom::Bloom, tonemapping::DebandDither},
    prelude::*,
};

use crate::components::{
    background::{BackgroundPluginSettings, RenderBackground},
    game_button::GameButton,
    mouse_event::Clicked,
};

use super::{AppSceneRoot, AppState};

pub struct MainMenuPlugin;

pub mod about;
pub mod settings;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup)
            .insert_resource(BackgroundPluginSettings {
                shader: "stars_main_menu.wgsl".to_string(),
            });
    }
}

fn setup(mut commands: Commands, root_entity: Res<AppSceneRoot>, asset_server: Res<AssetServer>) {
    commands
        .entity(root_entity.world)
        .with_child((
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
        ))
        .with_child((
            AudioPlayer::new(asset_server.load("Cojam - Milky Main Menu.ogg")),
            PlaybackSettings {
                mode: PlaybackMode::Loop,
                ..Default::default()
            },
        ));

    let spacer = Node {
        height: Val::Px(5.0),
        ..Default::default()
    };
    let new_game = commands
        .spawn(GameButton::new("New game", 200.0))
        .observe(
            |_: Trigger<Clicked>, mut next: ResMut<NextState<AppState>>| next.set(AppState::Game),
        )
        .id();
    let settings = commands
        .spawn(GameButton::new("Settings", 200.0))
        .observe(
            |_: Trigger<Clicked>, mut next: ResMut<NextState<AppState>>| {
                println!("clicked settings");
                next.set(AppState::MainMenuSettings)
            },
        )
        .id();
    // let about = commands
    //     .spawn(GameButton::new("About", 200.0))
    //     .observe(
    //         |_: Trigger<Clicked>, mut next: ResMut<NextState<AppState>>| {
    //             next.set(AppState::MainMenuAbout)
    //         },
    //     )
    //     .id();
    let exit = commands
        .spawn(GameButton::new("Exit", 0.0))
        .observe(
            |_: Trigger<Clicked>, mut app_exit_events: ResMut<Events<AppExit>>| {
                app_exit_events.send(AppExit::Success);
            },
        )
        .id();
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
        .add_children(&[new_game])
        .with_child(spacer.clone())
        .add_children(&[settings])
        .with_child(spacer.clone())
        .add_children(&[exit]);
    });
}
