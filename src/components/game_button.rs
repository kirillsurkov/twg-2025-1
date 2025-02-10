use bevy::prelude::*;

use super::mouse_event::Clicked;

pub struct GameButtonPlugin;

impl Plugin for GameButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (init_buttons, animate).chain());
    }
}

#[derive(Component)]
pub struct GameButton {
    name: &'static str,
    hue: f32,
    timer: f32,
}

impl GameButton {
    pub fn new(name: &'static str, hue: f32) -> Self {
        Self {
            name,
            hue,
            timer: 0.0,
        }
    }
}

#[derive(Component, Clone, Reflect)]
struct ButtonPart {
    order: f32,
}

impl ButtonPart {
    fn new(order: f32) -> Self {
        Self { order }
    }
}

fn init_buttons(mut commands: Commands, buttons: Query<(Entity, &GameButton), Added<GameButton>>) {
    let width = 300.0;
    let height = 100.0;

    let node_full = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..Default::default()
    };

    for (entity, button) in buttons.iter() {
        commands
            .entity(entity)
            .insert((
                Button,
                Node {
                    width: Val::Px(width),
                    height: Val::Px(height),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ButtonPart::new(2.0),
                BorderRadius::MAX,
            ))
            .with_children(|p| {
                p.spawn((node_full.clone(), ButtonPart::new(1.0), BorderRadius::MAX))
                    .with_children(|p| {
                        p.spawn((node_full.clone(), ButtonPart::new(0.0), BorderRadius::MAX))
                            .with_child((node_full.clone(), BorderRadius::MAX));
                    });
                p.spawn((
                    Text::new(button.name),
                    Node {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    TextFont {
                        font_size: 36.0,
                        ..Default::default()
                    },
                ));
            });
    }
}

fn animate(
    mut commands: Commands,
    mut interaction_query: Query<(Entity, Ref<Interaction>, &mut GameButton)>,
    mut nodes: Query<&mut Node>,
    mut border_colors: Query<&mut BorderColor>,
    parts: Query<&ButtonPart>,
    texts: Query<&Text>,
    children: Query<&Children>,
    time: Res<Time>,
) {
    let ease = EasingCurve::new(0.0, 1.0, EaseFunction::Linear);
    let speed = 20.0;

    for (entity, interaction, mut game_button) in interaction_query.iter_mut() {
        game_button.timer = match *interaction {
            Interaction::None => (game_button.timer - speed * time.delta_secs()).max(0.0),
            Interaction::Hovered => (game_button.timer + speed * time.delta_secs()).min(1.0),
            Interaction::Pressed => {
                if interaction.is_changed() {
                    commands.trigger_targets(Clicked, entity);
                }
                (game_button.timer + speed * time.delta_secs()).min(2.0)
            }
        };
        for entity in children.iter_descendants(entity).chain([entity]) {
            if let Ok(part) = parts.get(entity) {
                let Ok(mut node) = nodes.get_mut(entity) else {
                    continue;
                };
                let Ok(mut border_color) = border_colors.get_mut(entity) else {
                    continue;
                };
                let alpha = (1.0 * (part.order - game_button.timer).min(0.0).max(-1.0)).max(-1.0);
                let sample = ease
                    .sample((game_button.timer - part.order).max(0.0).min(1.0))
                    .unwrap();
                node.border = UiRect::all(Val::Px(2.0 * (1.0 - sample)));
                node.padding = UiRect::all(Val::Px(5.0 * (1.0 - sample)));
                border_color.0 = Color::hsva(
                    45.0,
                    0.75,
                    0.5,
                    (0.6 - 0.27 * (part.order - game_button.timer)) * (1.0 + alpha),
                );
            } else if texts.get(entity).is_err() {
                commands
                    .entity(entity)
                    .insert(BackgroundColor(match *interaction {
                        Interaction::None => Color::hsva(game_button.hue, 0.5, 0.75, 0.1),
                        Interaction::Hovered => Color::hsva(game_button.hue, 0.5, 1.0, 0.1),
                        Interaction::Pressed => Color::hsva(game_button.hue, 0.5, 1.0, 0.15),
                    }));
            }
        }
    }
}
