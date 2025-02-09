use bevy::prelude::*;

use crate::scenes::AppState;

use super::palette::COLOR_CONTAINER;

pub struct GameUiContainerPlugin;

impl Plugin for GameUiContainerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init.run_if(in_state(AppState::Game)));
    }
}

#[derive(Component)]
pub struct GameUiContainer;

#[derive(Component)]
enum State {
    Idle,
}

fn init(mut commands: Commands, containers: Query<(Entity, &GameUiContainer, Option<&State>)>) {
    for (entity, container, state) in containers.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    BackgroundColor(COLOR_CONTAINER),
                    BoxShadow {
                        blur_radius: Val::Px(5.0),
                        spread_radius: Val::Px(5.0),
                        x_offset: Val::ZERO,
                        y_offset: Val::Px(5.0),
                        color: Color::BLACK.with_alpha(0.5),
                    },
                    State::Idle,
                ));
            }
            _ => {}
        }
    }
}
