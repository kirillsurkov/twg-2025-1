use bevy::prelude::*;

use crate::scenes::{game::GameState, AppState};

use super::palette::{COLOR_POWER_HIGH, COLOR_POWER_LOW, COLOR_TEXT};

pub struct GameUiPowerBarPlugin;

impl Plugin for GameUiPowerBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (init, update)
                .chain()
                .run_if(in_state(AppState::Game).and(in_state(GameState::Idle))),
        );
    }
}

#[derive(Component)]
pub struct GameUiPowerBar {
    pub power: f32,
}

impl GameUiPowerBar {
    pub fn new() -> Self {
        Self { power: 1.0 }
    }
}

#[derive(Component)]
enum State {
    Idle(Entity),
}

fn init(mut commands: Commands, bars: Query<(Entity, &GameUiPowerBar, Option<&State>)>) {
    for (entity, bar, state) in bars.iter() {
        match state {
            None => {
                let indicator = commands
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..Default::default()
                        },
                        BackgroundColor(COLOR_POWER_HIGH),
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            column_gap: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        State::Idle(indicator),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new("Power:"),
                            TextColor(COLOR_TEXT),
                            TextFont {
                                font_size: 36.0,
                                ..Default::default()
                            },
                        ));
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    column_gap: Val::Px(10.0),
                                    ..Default::default()
                                },
                                BackgroundColor(Color::BLACK),
                                State::Idle(indicator),
                            ))
                            .add_child(indicator);
                    });
            }
            _ => {}
        }
    }
}

fn update(
    bars: Query<(&GameUiPowerBar, &State)>,
    mut indicators: Query<(&mut Node, &mut BackgroundColor)>,
) {
    for (bar, state) in bars.iter() {
        let State::Idle(indicator) = state;
        let Ok((mut node, mut color)) = indicators.get_mut(*indicator) else {
            continue;
        };
        node.width = Val::Percent(bar.power * 100.0);
        color.0 = COLOR_POWER_LOW.mix(&COLOR_POWER_HIGH, bar.power);
    }
}
