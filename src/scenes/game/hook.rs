use core::f32;

use bevy::{prelude::*, utils::HashSet};
use bevy_rapier2d::prelude::*;
use rand_distr::num_traits::Zero;

use crate::{
    components::{collisions::Collisions, material_modifier::MaterialModifier},
    scenes::AppState,
};

use super::{
    builder::{Enabled, Ready},
    game_cursor::{CursorLayer, GameCursor},
    map_state::MapState,
    player::PlayerState,
    rock::{Rock, RockState},
};

pub struct HookPlugin;

impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (init, update, user_interact)
                .chain()
                .run_if(in_state(AppState::Game)),
        );
    }
}

#[derive(Component)]
pub struct Hook(pub bool);

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
    Done {
        body: Entity,
        head: Entity,
        radar: Entity,
    },
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
                let radar = commands
                    .spawn((
                        Transform::default(),
                        Collider::ball(9.5),
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
                        Ready,
                        LoadingState::Done { body, head, radar },
                        HookState::Idle,
                    ))
                    .add_children(&[body, head, radar]);
            }
            _ => {}
        }
    }
}

fn user_interact(
    mut commands: Commands,
    player_state: Res<State<PlayerState>>,
    game_cursor: Option<Res<GameCursor>>,
    mut hooks: Query<
        (Entity, &LoadingState, &mut HookState, &GlobalTransform),
        (With<Hook>, With<Enabled>),
    >,
) {
    let Some(game_cursor) = game_cursor else {
        return;
    };

    let PlayerState::Interact(px, py) = *player_state.get() else {
        return;
    };

    for (entity, loading_state, mut hook_state, transform) in hooks.iter_mut() {
        match loading_state {
            LoadingState::Done { .. } => {}
        }

        let IVec2 { x, y } = GameCursor::world_to_game(
            transform.translation().x,
            transform.translation().y,
            CursorLayer::Room,
        );

        if x != px || y != py {
            continue;
        }

        commands
            .entity(entity)
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
}

fn update(
    mut commands: Commands,
    mut hooks: Query<(&Hook, &LoadingState, &mut HookState), (With<Hook>, With<Enabled>)>,
    mut hook_bodies: Query<(&GlobalTransform, &mut Visibility), Without<RockState>>,
    mut rocks: Query<(&Rock, &mut RockState, &mut Transform)>,
    mut transforms: Query<&mut Transform, Without<RockState>>,
    mut targeted_rocks: Local<HashSet<Entity>>,
    mut map_state: ResMut<MapState>,
    collisions: Res<Collisions>,
    time: Res<Time>,
) {
    for (Hook(automatic), loading_state, mut hook_state) in hooks.iter_mut() {
        let LoadingState::Done { body, head, radar } = *loading_state;

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

        let origin = global_transform.translation().xy();

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
                        if let Ok((rock, _, _)) = rocks.get(rock) {
                            for (cargo, count) in rock.resources() {
                                map_state.harvest(cargo, count);
                            }
                        }
                        if let Some(rock) = commands.get_entity(rock) {
                            targeted_rocks.remove(&rock.id());
                            rock.despawn_recursive();
                        }
                    }
                    length = 0.0;
                    HookState::Idle
                } else {
                    if let Some(rock) = rock {
                        if let Ok((_, _, mut transform)) = rocks.get_mut(rock) {
                            transform.translation =
                                (origin + dir * (length + 1.0) * 0.5).extend(2.0);
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

        match *hook_state {
            HookState::Flying { dir, length } => {
                for rock in collisions.get(head) {
                    if let Ok((_, mut rock_state, _)) = rocks.get_mut(*rock) {
                        let RockState::Idle = *rock_state else {
                            continue;
                        };
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
            HookState::Idle if *automatic => {
                let mut target: Option<(Entity, Vec2)> = None;
                for entity in collisions.get(radar) {
                    if targeted_rocks.contains(entity) {
                        continue;
                    }
                    if let Ok((rock, rock_state, transform)) = rocks.get(*entity) {
                        let RockState::Idle = *rock_state else {
                            continue;
                        };
                        let delta = transform.translation.xy() - origin;
                        let a = rock.movement_speed.length_squared() - speed * speed;
                        let b = 2.0 * delta.length() * rock.movement_speed.length();
                        let c = delta.length_squared();
                        let time = if a.is_zero() {
                            (c / b).abs()
                        } else {
                            let d = b * b - 4.0 * a * c;
                            ((d.sqrt() - b) / (2.0 * a)).abs()
                        };
                        let intersection = transform.translation.xy() + rock.movement_speed * time;
                        target = if let Some((_, pos)) = target {
                            let distance1 = pos.distance_squared(origin);
                            let distance2 = intersection.distance_squared(origin);
                            if distance1 < distance2 {
                                Some((*entity, intersection))
                            } else {
                                None
                            }
                        } else {
                            Some((*entity, intersection))
                        };
                    }
                }
                if let Some((entity, pos)) = target {
                    if let Ok(dir) = Dir2::new(pos - origin) {
                        targeted_rocks.insert(entity);
                        *hook_state = HookState::Flying { dir, length: 0.0 }
                    }
                }
            }
            _ => {}
        }
    }
}
