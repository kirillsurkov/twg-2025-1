use bevy::prelude::*;

use crate::{components::clicked_event::Clicked, scenes::AppState};

use super::palette::{COLOR_HIGHLIGHT_DARK, COLOR_HIGHLIGHT_LIGHT, COLOR_TEXT};

pub struct GameUiContainerItemPlugin;

impl Plugin for GameUiContainerItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (init, update).chain().run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct GameUiContainerItem {
    title: String,
    is_button: bool,
    child_spawners: Vec<Box<dyn FnOnce(&mut ChildBuilder) + Send + Sync>>,
}

impl GameUiContainerItem {
    pub fn new<T: ToString>(title: T) -> Self {
        Self {
            title: title.to_string(),
            is_button: false,
            child_spawners: vec![],
        }
    }

    pub fn button(mut self) -> Self {
        self.is_button = true;
        self
    }

    pub fn footer(mut self, bundle: impl Bundle) -> Self {
        self.child_spawners
            .push(Box::new(move |builder: &mut ChildBuilder| {
                builder.spawn(Node::default()).insert(bundle);
            }));
        self
    }
}

#[derive(Component)]
enum State {
    Idle,
}

fn init(
    mut commands: Commands,
    mut containers: Query<(Entity, &mut GameUiContainerItem, Option<&State>)>,
) {
    for (entity, mut item, state) in containers.iter_mut() {
        match state {
            None => {
                let mut entity = commands.entity(entity);
                entity.insert((
                    Node {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(10.0),
                        ..Default::default()
                    },
                    State::Idle,
                ));
                if item.is_button {
                    entity.insert(Button);
                }
                entity.with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(80.0),
                            flex_shrink: 0.0,
                            ..Default::default()
                        },
                        BackgroundColor(Color::BLACK),
                    ));
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            ..Default::default()
                        })
                        .with_child((
                            Text(item.title.clone()),
                            TextColor(COLOR_TEXT),
                            TextFont {
                                font_size: 32.0,
                                ..Default::default()
                            },
                        ));
                    for spawner in item.child_spawners.drain(..) {
                        spawner(parent);
                    }
                });
            }
            _ => {}
        }
    }
}

fn update(
    mut commands: Commands,
    interactions: Query<(Entity, Ref<Interaction>), With<GameUiContainerItem>>,
) {
    for (entity, interaction) in interactions.iter() {
        match *interaction {
            Interaction::None => {
                commands.entity(entity).remove::<BackgroundColor>();
            }
            Interaction::Hovered => {
                commands
                    .entity(entity)
                    .insert(BackgroundColor(COLOR_HIGHLIGHT_DARK));
            }
            Interaction::Pressed => {
                if interaction.is_changed() {
                    commands.trigger_targets(Clicked, entity);
                }
                commands
                    .entity(entity)
                    .insert(BackgroundColor(COLOR_HIGHLIGHT_LIGHT));
            }
        }
    }
}
