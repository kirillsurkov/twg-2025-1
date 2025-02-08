use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::{collisions::Collisions, material_modifier::MaterialModifier},
    scenes::AppState,
};

use super::{
    game_cursor::{update_cursor, CursorLayer, GameCursor},
    player::{PlayerInteractEntity, PlayerState},
    rock::RockState,
};

pub struct HookPlugin;

impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init,
                update,
                (
                    user_interact.run_if(
                        in_state(PlayerState::Interact)
                            .and(resource_exists::<PlayerInteractEntity>),
                    ),
                    set_state.run_if(in_state(PlayerState::Idle)),
                )
                    .chain()
                    .run_if(resource_exists::<GameCursor>)
                    .after(update_cursor),
            )
                .run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct Hook;

#[derive(Component)]
enum HookState {
    Idle,
    Flying {
        dir: Dir2,
        length: f32,
    },
    Returning {
        dir: Dir2,
        length: f32,
        rock: Option<Entity>,
    },
}

#[derive(Component, PartialEq)]
enum LoadingState {
    Done { body: Entity, head: Entity },
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hooks: Query<(Entity, Option<&LoadingState>), With<Hook>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, state) in hooks.iter() {
        match state {
            None => {
                let body = commands
                    .spawn((
                        Mesh3d(meshes.add(Cylinder::new(0.1, 1.0))),
                        MeshMaterial3d(materials.add(StandardMaterial::default())),
                        Visibility::Hidden,
                    ))
                    .id();
                let head = commands
                    .spawn((
                        SceneRoot(
                            asset_server.load(GltfAssetLabel::Scene(0).from_asset("hook_head.glb")),
                        ),
                        Collider::ball(0.33),
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::STATIC_STATIC,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert((
                        SceneRoot(
                            asset_server.load(GltfAssetLabel::Scene(0).from_asset("hook_base.glb")),
                        ),
                        LoadingState::Done { body, head },
                        HookState::Idle,
                    ))
                    .add_children(&[body, head]);
            }
            _ => {}
        }
    }
}

fn set_state(
    mut commands: Commands,
    mut next_player_state: ResMut<NextState<PlayerState>>,
    game_cursor: Option<Res<GameCursor>>,
    hooks: Query<(Entity, &LoadingState, &GlobalTransform), With<Hook>>,
) {
    // WTF
    let Some(game_cursor) = game_cursor else {
        return;
    };

    for (entity, loading_state, transform) in hooks.iter() {
        match loading_state {
            LoadingState::Done { .. } => {}
        }

        let pos = GameCursor::world_to_game(
            transform.translation().x,
            transform.translation().y,
            CursorLayer::Room,
        );
        let is_selected = game_cursor.x == pos.x && game_cursor.y == pos.y;

        let mut entity = commands.entity(entity);
        entity.remove::<MaterialModifier<StandardMaterial>>();

        if is_selected {
            entity.insert(MaterialModifier::new(move |mut mat: StandardMaterial| {
                mat.base_color = mat.base_color.lighter(0.05);
                mat
            }));
        }

        let entity = entity.id();

        if is_selected && game_cursor.just_pressed {
            next_player_state.set(PlayerState::Interact);
            commands.insert_resource(PlayerInteractEntity(entity));
        }
    }
}

fn user_interact(
    mut commands: Commands,
    game_cursor: Option<Res<GameCursor>>,
    interact: Res<PlayerInteractEntity>,
    mut hooks: Query<(&LoadingState, &mut HookState, &GlobalTransform), With<Hook>>,
) {
    // WTF
    let Some(game_cursor) = game_cursor else {
        return;
    };

    let (loading_state, mut hook_state, transform) = hooks.get_mut(interact.0).unwrap();

    match loading_state {
        LoadingState::Done { .. } => {}
    }

    commands
        .entity(interact.0)
        .remove::<MaterialModifier<StandardMaterial>>()
        .insert(MaterialModifier::new(|mut mat: StandardMaterial| {
            mat.base_color = mat.base_color.lighter(0.1);
            mat
        }));

    match *hook_state {
        HookState::Idle if game_cursor.just_pressed => {
            let cursor_pos = Vec2::new(game_cursor.fx, game_cursor.fy);
            let hook_pos = transform.translation().xy();
            let Ok(dir) = Dir2::new(cursor_pos - hook_pos) else {
                return;
            };
            *hook_state = HookState::Flying { dir, length: 0.0 };
        }
        _ => {}
    }
}

fn update(
    mut commands: Commands,
    mut hooks: Query<(&LoadingState, &mut HookState), With<Hook>>,
    mut hook_bodies: Query<(&GlobalTransform, &mut Visibility), Without<RockState>>,
    mut rocks: Query<(&mut RockState, &mut Transform)>,
    mut transforms: Query<&mut Transform, Without<RockState>>,
    collisions: Res<Collisions>,
    time: Res<Time>,
) {
    for (loading_state, mut hook_state) in hooks.iter_mut() {
        let LoadingState::Done { body, head } = *loading_state;

        let Ok((global_transform, mut visibility)) = hook_bodies.get_mut(body) else {
            continue;
        };

        let Ok([mut body_transform, mut head_transform]) = transforms.get_many_mut([body, head])
        else {
            continue;
        };

        *visibility = Visibility::Inherited;

        let speed = 15.0;
        let max_length = 10.0;

        let origin = global_transform.translation();

        let (dir, length) = match *hook_state {
            HookState::Idle => {
                *visibility = Visibility::Hidden;
                (Dir2::NORTH, 0.0)
            }
            HookState::Flying { dir, mut length } => {
                length += time.delta_secs() * speed;
                *hook_state = if length >= max_length {
                    length = max_length;
                    HookState::Returning {
                        dir,
                        length,
                        rock: None,
                    }
                } else {
                    HookState::Flying { dir, length }
                };
                (dir, length)
            }
            HookState::Returning {
                dir,
                mut length,
                rock,
            } => {
                length -= time.delta_secs() * speed;
                *hook_state = if length <= 0.0 {
                    if let Some(rock) = rock {
                        commands.entity(rock).despawn_recursive();
                    }
                    length = 0.0;
                    HookState::Idle
                } else {
                    if let Some(rock) = rock {
                        if let Ok((_, mut transform)) = rocks.get_mut(rock) {
                            transform.translation = origin + dir.extend(0.0) * (length + 1.0) * 0.5;
                        }
                    }
                    HookState::Returning { dir, length, rock }
                };
                (dir, length)
            }
        };

        body_transform.rotation = Quat::from_rotation_arc(Vec3::Y, dir.extend(0.0));
        body_transform.scale.y = length;
        body_transform.translation = dir.extend(0.0) * length * 0.5;
        body_transform.translation.z = 2.0;

        head_transform.rotation = Quat::from_rotation_arc(Vec3::Z, dir.extend(0.0));
        head_transform.translation = dir.extend(0.0) * length.max(0.2);
        head_transform.translation.z = 2.0;

        if let HookState::Flying { dir, length } = *hook_state {
            for rock in collisions.get(head) {
                if let Ok((mut rock_state, _)) = rocks.get_mut(*rock) {
                    *hook_state = HookState::Returning {
                        dir,
                        length,
                        rock: Some(*rock),
                    };
                    *rock_state = RockState::Hooked;
                    break;
                }
            }
        }
    }
}
