use bevy::prelude::*;
use game::GamePlugin;
use main_menu::MainMenuPlugin;

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
            .add_plugins(GamePlugin)
            .add_systems(OnExit(AppState::Game), cleanup);
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum AppState {
    Splash,
    MainMenu,
    Game,
    Titles,
}

#[derive(Resource)]
pub struct AppSceneRoot {
    world: Entity,
    ui: Entity,
}

fn cleanup(mut commands: Commands, root_entity: Res<AppSceneRoot>) {
    println!("cleanup");
    commands.entity(root_entity.world).despawn_descendants();
    commands.entity(root_entity.ui).despawn_descendants();
}
