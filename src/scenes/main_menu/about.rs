use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::DebandDither},
    prelude::*,
};

use crate::components::{
    background::RenderBackground, game_button::GameButton, mouse_event::Clicked,
};

use super::{AppSceneRoot, AppState};

pub struct MainMenuAboutPlugin;

impl Plugin for MainMenuAboutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenuAbout), setup);
    }
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
    let back = commands
        .spawn(GameButton::new("Back", 200.0))
        .observe(
            |_: Trigger<Clicked>, mut next: ResMut<NextState<AppState>>| {
                next.set(AppState::MainMenu)
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
        .with_child(spacer.clone())
        .add_children(&[back]);
    });
}
