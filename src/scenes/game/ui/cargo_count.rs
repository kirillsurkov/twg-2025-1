use bevy::prelude::*;

use crate::scenes::AppState;

use super::palette::COLOR_TEXT;

pub struct GameUiCargoCountPlugin;

impl Plugin for GameUiCargoCountPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (init, update).chain().run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct GameUiCargoCount {
    pub cur: f32,
    pub max: f32,
}

impl GameUiCargoCount {
    pub fn new(cur: f32, max: f32) -> Self {
        Self { cur, max }
    }
}

#[derive(Component)]
enum State {
    Idle { cur: Entity, max: Entity },
}

fn init(
    mut commands: Commands,
    containers: Query<(Entity, Option<&State>), With<GameUiCargoCount>>,
) {
    for (entity, state) in containers.iter() {
        match state {
            None => {
                let cur = commands
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            ..Default::default()
                        },
                        Text::default(),
                        TextLayout::new_with_justify(JustifyText::Left),
                        TextColor(COLOR_TEXT),
                        TextFont {
                            font_size: 20.0,
                            ..Default::default()
                        },
                    ))
                    .id();
                let max = commands
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            ..Default::default()
                        },
                        Text::default(),
                        TextLayout::new_with_justify(JustifyText::Right),
                        TextColor(COLOR_TEXT),
                        TextFont {
                            font_size: 20.0,
                            ..Default::default()
                        },
                    ))
                    .id();
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
                        State::Idle { cur, max },
                    ))
                    .add_child(cur)
                    .with_child((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(3.0),
                            flex_shrink: 0.0,
                            ..Default::default()
                        },
                        BackgroundColor(COLOR_TEXT),
                    ))
                    .add_child(max);
            }
            _ => {}
        }
    }
}

fn update(counts: Query<(&GameUiCargoCount, &State)>, mut texts: Query<&mut Text>) {
    for (count, state) in counts.iter() {
        let State::Idle { cur, max } = state;
        if let Ok(mut text) = texts.get_mut(*cur) {
            text.0 = format!("{:.2}", count.cur);
        }
        if let Ok(mut text) = texts.get_mut(*max) {
            text.0 = format!("{:.2}", count.max);
        }
    }
}
