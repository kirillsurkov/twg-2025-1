use bevy::prelude::*;

use crate::scenes::AppState;

use super::palette::{COLOR_CONTAINER, COLOR_HEADER, COLOR_TEXT};

pub struct GameUiHeaderPlugin;

impl Plugin for GameUiHeaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init.run_if(in_state(AppState::Game)));
    }
}

#[derive(Component)]
pub struct GameUiHeader {
    title: String,
}

impl GameUiHeader {
    pub fn new<T: ToString>(title: T) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

#[derive(Component)]
enum State {
    Idle,
}

fn init(mut commands: Commands, headers: Query<(Entity, &GameUiHeader, Option<&State>)>) {
    for (entity, header, state) in headers.iter() {
        match state {
            None => {
                commands
                    .entity(entity)
                    .insert((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(80.0),
                            flex_shrink: 0.0,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        BackgroundColor(COLOR_HEADER),
                        State::Idle,
                    ))
                    .with_child((
                        Text(header.title.clone()),
                        TextColor(COLOR_TEXT),
                        TextFont {
                            font_size: 36.0,
                            ..Default::default()
                        },
                    ));
            }
            _ => {}
        }
    }
}
