use bevy::{audio::PlaybackMode, prelude::*};
use rand::seq::IteratorRandom;

use crate::scenes::{AppSceneRoot, AppState};

pub struct MusicPlayerPlugin;

impl Plugin for MusicPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update.run_if(in_state(AppState::Game)));
    }
}

#[derive(Component)]
struct Music;

fn update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    music: Query<(), With<Music>>,
    root_entity: Res<AppSceneRoot>,
) {
    if music.is_empty() {
        let files = std::fs::read_dir("./assets/music").unwrap();
        let file = files.choose(&mut rand::rng()).unwrap().unwrap();
        commands.entity(root_entity.world).with_child((
            Music,
            AudioPlayer::new(
                asset_server.load(format!("music/{}", file.file_name().into_string().unwrap())),
            ),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..Default::default()
            },
        ));
    }
}
