use bevy::prelude::*;

use crate::scenes::AppState;

use super::palette::COLOR_TEXT;

pub struct GameUiCargoCountPlugin;

impl Plugin for GameUiCargoCountPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init.run_if(in_state(AppState::Game)));
    }
}

#[derive(Component)]
pub struct GameUiCargoCount {
    count: u32,
    max: u32,
}

impl GameUiCargoCount {
    pub fn new(count: u32, max: u32) -> Self {
        Self { count, max }
    }
}

#[derive(Component)]
enum State {
    Idle,
}

fn init(mut commands: Commands, containers: Query<(Entity, &GameUiCargoCount, Option<&State>)>) {
    for (entity, item, state) in containers.iter() {
        match state {
            None => {
                commands
                    .entity(entity)
                    .insert((
                        Node {
                            width: Val::Px(100.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        State::Idle,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text(format!("{}", item.count)),
                            TextColor(COLOR_TEXT),
                            TextFont {
                                font_size: 24.0,
                                ..Default::default()
                            },
                        ));
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(3.0),
                                flex_shrink: 0.0,
                                ..Default::default()
                            },
                            BackgroundColor(COLOR_TEXT),
                        ));
                        parent.spawn((
                            Text(format!("{}", item.max)),
                            TextColor(COLOR_TEXT),
                            TextFont {
                                font_size: 24.0,
                                ..Default::default()
                            },
                        ));
                    });
            }
            _ => {}
        }
    }
}
