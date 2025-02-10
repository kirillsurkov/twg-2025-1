use bevy::prelude::*;
use game::GamePlugin;
use main_menu::{about::MainMenuAboutPlugin, settings::MainMenuSettingsPlugin, MainMenuPlugin};

pub mod game;
pub mod main_menu;

pub struct AppScenesPlugin;

impl Plugin for AppScenesPlugin {
    fn build(&self, app: &mut App) {
        let root_world = app
            .world_mut()
            .spawn((Name::new("root world"), Transform::default()))
            .id();
        let root_ui = app
            .world_mut()
            .spawn((
                Name::new("root ui"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
            ))
            .id();
        app.insert_state(AppState::MainMenu)
            .insert_resource(AppSceneRoot {
                world: root_world,
                ui: root_ui,
            })
            .add_plugins(MainMenuPlugin)
            .add_systems(OnExit(AppState::MainMenu), cleanup)
            .add_plugins(MainMenuSettingsPlugin)
            .add_systems(OnExit(AppState::MainMenuSettings), cleanup)
            .add_plugins(MainMenuAboutPlugin)
            .add_systems(OnExit(AppState::MainMenuAbout), cleanup)
            .add_plugins(GamePlugin)
            .add_systems(OnExit(AppState::Game), cleanup);
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AppState {
    Splash,
    MainMenu,
    MainMenuSettings,
    MainMenuAbout,
    Game,
    Titles,
}

#[derive(Resource)]
pub struct AppSceneRoot {
    pub world: Entity,
    pub ui: Entity,
}

fn cleanup(
    mut commands: Commands,
    root_entity: Res<AppSceneRoot>,
    audio_sinks: Query<&AudioSink>,
    children: Query<&Children>,
) {
    println!("cleanup");

    for entity in children.iter_descendants(root_entity.world) {
        if let Ok(sink) = audio_sinks.get(entity) {
            sink.stop();
        }
    }

    for entity in children.iter_descendants(root_entity.ui) {
        if let Ok(sink) = audio_sinks.get(entity) {
            sink.stop();
        }
    }

    commands.entity(root_entity.world).despawn_descendants();
    commands.entity(root_entity.ui).despawn_descendants();
}
