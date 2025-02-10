use std::f32::consts::{FRAC_PI_8, SQRT_2, TAU};

use bevy::{
    gltf::GltfMaterialName, prelude::*, render::render_resource::ShaderType, utils::HashMap,
};
use bevy_rapier2d::prelude::*;
use ops::FloatPow;
use rand::Rng;
use rand_distr::{weighted::WeightedIndex, Distribution};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    components::{
        collisions::Collisions,
        procedural_material::{ProceduralMaterial, ProceduralMaterialPlugin},
    },
    scenes::{AppSceneRoot, AppState},
    RandomRotation,
};

use super::{
    game_cursor::{CursorLayer, GameCursor},
    map_state::{Cargo, MapLayer, MapState},
    room::Room,
};

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RockMaterial>::default())
            .add_systems(
                Update,
                (init, update_pos, rock_spawner.after(init)).run_if(in_state(AppState::Game)),
            );
    }
}

#[derive(Clone, Copy, EnumIter)]
enum RockKind {
    Silicon,
    Ice,
    Copper,
    Uranium,
    Aurelium,
}

impl RockKind {
    fn probability(self) -> f32 {
        match self {
            RockKind::Silicon => 0.5,
            RockKind::Copper => 0.25,
            RockKind::Ice => 0.20,
            RockKind::Uranium => 0.05,
            RockKind::Aurelium => 0.00,
        }
    }

    fn color(self) -> Color {
        match self {
            RockKind::Silicon => Color::NONE,
            RockKind::Copper => Color::srgba(1.0, 0.6, 0.0, 1.0),
            RockKind::Ice => Color::srgba(0.0, 1.0, 1.0, 1.0),
            RockKind::Uranium => Color::srgba(0.0, 1.0, 0.0, 1.0),
            RockKind::Aurelium => Color::srgba(1.0, 1.0, 0.0, 1.0),
        }
    }

    fn resources(self) -> HashMap<Cargo, f32> {
        match self {
            RockKind::Silicon => vec![(Cargo::Stone, 1.0), (Cargo::Silicon, 5.0)],
            RockKind::Copper => vec![(Cargo::Copper, 1.0), (Cargo::Silicon, 5.0)],
            RockKind::Ice => vec![(Cargo::Ice, 1.0), (Cargo::Silicon, 5.0)],
            RockKind::Uranium => vec![(Cargo::Uranium, 1.0), (Cargo::Silicon, 5.0)],
            RockKind::Aurelium => vec![(Cargo::Aurelium, 1.0), (Cargo::Silicon, 5.0)],
        }
        .into_iter()
        .collect()
    }
}

#[derive(Component)]
pub struct Rock {
    pub movement_speed: Vec2,
    rotation_speed: f32,
    rotation_axis: Dir3,
    scale: f32,
    kind: RockKind,
}

impl Rock {
    pub fn resources(&self) -> HashMap<Cargo, f32> {
        self.kind
            .resources()
            .into_iter()
            .map(|(k, v)| (k, v * self.scale))
            .collect()
    }
}

#[derive(Component, PartialEq)]
pub enum RockState {
    Idle,
    Hooked,
}

#[derive(Component, PartialEq)]
enum LoadingState {
    Materials,
    Done,
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rocks: Query<(Entity, &Rock, Option<&LoadingState>)>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
    mesh_handles: Query<&Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    for (entity, rock, state) in rocks.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("rock_0.glb"))),
                    LoadingState::Materials,
                    RockState::Idle,
                    Visibility::Hidden,
                    Collider::ball(1.0),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::STATIC_STATIC,
                ));
            }
            Some(LoadingState::Materials) => {
                for child in children.iter_descendants(entity) {
                    if !gltf_materials
                        .get(child)
                        .map_or(false, |m| m.0 == "rock_material")
                    {
                        continue;
                    }

                    let mesh = meshes.get(mesh_handles.get(child).unwrap()).unwrap();
                    let triangles = mesh.triangles().unwrap().collect::<Vec<_>>();
                    let weighted = WeightedIndex::new(triangles.iter().map(|t| t.area())).unwrap();

                    for _ in 0..5 {
                        let triangle = triangles[weighted.sample(&mut rng)];
                        let size = rng.random_range(0.1..0.5);
                        let height = rng.random_range(1.0..1.5);

                        let u = rng.random::<f32>();
                        let v = rng.random::<f32>();
                        let (u, v) = if u + v < 1.0 {
                            (u, v)
                        } else {
                            (1.0 - u, 1.0 - v)
                        };
                        let spawn_point = triangle.vertices[0]
                            + u * (triangle.vertices[1] - triangle.vertices[0])
                            + v * (triangle.vertices[2] - triangle.vertices[0]);

                        let ang1 = rng.random_range(-FRAC_PI_8..FRAC_PI_8);
                        let ang2 = rng.random_range(-FRAC_PI_8..FRAC_PI_8);
                        let up = triangle.normal().unwrap();
                        let (right, forward) = up.any_orthonormal_pair();
                        let dir = ang1.sin() * right
                            + ang2.sin() * forward
                            + (1.0 - ang1.sin().squared() - ang2.sin().squared()).sqrt() * up;

                        commands.entity(entity).with_child((
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: rock.kind.color(),
                                unlit: true,
                                alpha_mode: AlphaMode::Blend,
                                metallic: 1.0,
                                perceptual_roughness: 0.5,
                                ..Default::default()
                            })),
                            Mesh3d(meshes.add(Cuboid::new(size, size, height))),
                            Transform::from_translation(spawn_point - dir * height * 0.25)
                                .looking_to(dir, Vec3::Z),
                        ));
                    }

                    commands
                        .entity(child)
                        .remove::<MeshMaterial3d<StandardMaterial>>()
                        .insert(RockMaterial::new(rand::random::<f32>() * 1000.0));
                    commands
                        .entity(entity)
                        .insert(LoadingState::Done)
                        .insert(Visibility::Inherited);
                }
            }
            Some(LoadingState::Done { .. }) => {}
        }
    }
}

fn update_pos(
    mut commands: Commands,
    mut rocks: Query<(Entity, &Rock, &RockState, &mut Transform), Without<Room>>,
    mut map_state: ResMut<MapState>,
    rooms: Query<&Transform, With<Room>>,
    collisions: Res<Collisions>,
    time: Res<Time>,
) {
    let (min, max) = map_state.get_bounds();
    let min = Vec2::from(GameCursor::game_to_world(min.x, min.y, CursorLayer::Room)) - 40.0;
    let max = Vec2::from(GameCursor::game_to_world(max.x, max.y, CursorLayer::Room)) + 40.0;

    for (entity, rock, rock_state, mut transform) in rocks.iter_mut() {
        if transform.translation.x <= min.x
            || transform.translation.x >= max.x
            || transform.translation.y <= min.y
            || transform.translation.y >= max.y
        {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        if *rock_state != RockState::Idle {
            continue;
        }

        transform.translation += rock.movement_speed.extend(0.0) * time.delta_secs();
        transform.rotate_axis(
            rock.rotation_axis,
            TAU * rock.rotation_speed * time.delta_secs(),
        );

        for room in collisions.get(entity) {
            if let Ok(transform) = rooms.get(*room) {
                let IVec2 { x, y } = GameCursor::world_to_game(
                    transform.translation.x,
                    transform.translation.y,
                    CursorLayer::Room,
                );
                if map_state.is_room(x, y, MapLayer::Main) {
                    map_state.remove_room(x, y, MapLayer::Main);
                    commands.entity(entity).try_despawn_recursive();
                }
            }
        }
    }
}

fn ray_intersects_circle(
    ray_origin: Vec2,
    ray_dir: Vec2,
    circle_center: Vec2,
    radius: f32,
) -> bool {
    let oc = circle_center - ray_origin;

    let t_closest = oc.dot(ray_dir);
    let closest_point = Vec2 {
        x: ray_origin.x + t_closest * ray_dir.x,
        y: ray_origin.y + t_closest * ray_dir.y,
    };

    let dist_to_center_sq = (closest_point.x - circle_center.x).squared()
        + (closest_point.y - circle_center.y).squared();

    if dist_to_center_sq > radius * radius {
        return false;
    }

    let offset = (radius * radius - dist_to_center_sq).sqrt();

    t_closest - offset >= 0.0 || t_closest + offset >= 0.0
}

fn rock_spawner(
    mut commands: Commands,
    root_entity: Res<AppSceneRoot>,
    map_state: Res<MapState>,
    time: Res<Time>,
    mut last_spawned: Local<f32>,
    mut last_spawned_aurelium: Local<f32>,
) {
    let mut rng = rand::rng();

    let (min, max) = map_state.get_bounds();
    let min = GameCursor::game_to_world(min.x, min.y, CursorLayer::Room);
    let max = GameCursor::game_to_world(max.x, max.y, CursorLayer::Room);

    let room_radius = 0.5 * (1.0 + CursorLayer::Room.size() * SQRT_2);

    let mut rocks = vec![];

    if time.elapsed_secs() - *last_spawned_aurelium >= 180.0 {
        let spawn_point1 = Vec2::new(
            rng.random_range(min.x - room_radius..=max.x + room_radius),
            max.y + 30.0,
        );
        let flight_dir1 = Vec2::NEG_Y;

        let spawn_point2 = Vec2::new(
            rng.random_range(min.x - room_radius..=max.x + room_radius),
            min.y - 30.0,
        );
        let flight_dir2 = Vec2::Y;

        let mut success = true;
        for IVec2 { x, y } in map_state.primary_blocks() {
            let room_pos = GameCursor::game_to_world(x, y, CursorLayer::Room);
            if ray_intersects_circle(spawn_point1, flight_dir1, room_pos, room_radius)
                || ray_intersects_circle(spawn_point2, flight_dir2, room_pos, room_radius)
            {
                success = false;
                break;
            }
        }

        let speed = 10.0;

        if success {
            *last_spawned_aurelium = time.elapsed_secs();
            rocks.push((spawn_point1, flight_dir1 * speed, RockKind::Aurelium));
            rocks.push((spawn_point2, flight_dir2 * speed, RockKind::Aurelium));
        }
    }

    if time.elapsed_secs() - *last_spawned >= 1.0 {
        let rand_y = rng.random_range(min.y - 10.0..=max.y + 10.0);
        let flight_dir = Vec2::NEG_X;

        let spawn_point = Vec2::new(max.x + 30.0, rand_y);

        let mut success = true;
        for IVec2 { x, y } in map_state.primary_blocks() {
            let room_pos = GameCursor::game_to_world(x, y, CursorLayer::Room);
            if ray_intersects_circle(spawn_point, flight_dir, room_pos, room_radius) {
                success = false;
                break;
            }
        }

        let speed = 2.0;

        let weighted = WeightedIndex::new(RockKind::iter().map(RockKind::probability)).unwrap();
        let kind = RockKind::iter().nth(weighted.sample(&mut rng)).unwrap();

        if success {
            *last_spawned = time.elapsed_secs();
            rocks.push((spawn_point, flight_dir * speed, kind));
        }
    }

    for (spawn_point, movement_speed, kind) in rocks {
        let scale = rng.random_range(0.2..0.5);
        commands.entity(root_entity.world).with_child((
            Rock {
                movement_speed,
                rotation_speed: rng.random_range(-1.0..1.0),
                rotation_axis: Transform::from_rotation(Quat::random()).forward(),
                scale,
                kind,
            },
            Transform::from_xyz(spawn_point.x, spawn_point.y, 2.0)
                .looking_to(movement_speed.normalize().extend(0.0), Vec3::Z)
                .with_scale(Vec3::splat(scale)),
        ));
    }
}

#[derive(Component, ShaderType, Clone)]
struct RockMaterial {
    seed: f32,
    time: f32,
    time_multiplier: f32,
}

impl RockMaterial {
    fn new(seed: f32) -> Self {
        Self {
            seed,
            time: 0.0,
            time_multiplier: 1.0,
        }
    }
}

impl ProceduralMaterial for RockMaterial {
    fn shader() -> &'static str {
        "rock.wgsl"
    }

    fn size() -> (u32, u32) {
        (64, 64)
    }
}
