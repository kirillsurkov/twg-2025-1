use std::f32::consts::{FRAC_PI_4, FRAC_PI_8, SQRT_2, TAU};

use bevy::{
    gltf::GltfMaterialName, prelude::*, render::render_resource::ShaderType, utils::HashMap,
};
use ops::FloatPow;
use rand::Rng;
use rand_distr::{weighted::WeightedIndex, Distribution};

use crate::{
    components::procedural_material::{ProceduralMaterial, ProceduralMaterialPlugin},
    RandomRotation,
};

use super::{
    game_cursor::{CursorLayer, GameCursor},
    map_state::{MapLayer, MapState},
};

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProceduralMaterialPlugin::<RockMaterial>::default())
            .add_systems(Update, (init, update_pos, rock_spawner.after(init)))
            .insert_resource(RockMapState {
                rocks: HashMap::new(),
            });
    }
}

#[derive(Resource)]
pub struct RockMapState {
    rocks: HashMap<IVec2, Entity>,
}

impl RockMapState {
    fn add(&mut self, x: i32, y: i32, entity: Entity) {
        self.rocks.insert(IVec2::new(x, y), entity);
    }

    pub fn rock(&self, x: i32, y: i32) -> Option<Entity> {
        self.rocks.get(&IVec2::new(x, y)).cloned()
    }
}

#[derive(Component)]
pub struct Rock {
    movement_speed: Vec3,
    rotation_speed: f32,
    rotation_axis: Dir3,
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
    rocks: Query<(Entity, Option<&LoadingState>), With<Rock>>,
    children: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
    mesh_handles: Query<&Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    for (entity, state) in rocks.iter() {
        match state {
            None => {
                commands.entity(entity).insert((
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("rock_0.glb"))),
                    LoadingState::Materials,
                    RockState::Idle,
                    Visibility::Hidden,
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
                                base_color: Color::srgba(1.0, 0.6, 0.0, 1.0),
                                emissive: LinearRgba::new(1.0, 0.4, 0.0, 1.0) * 0.0,
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
    mut rocks: Query<(Entity, &Rock, &RockState, &mut Transform)>,
    mut map_state: ResMut<MapState>,
    mut rock_map_state: ResMut<RockMapState>,
    time: Res<Time>,
) {
    let (min, max) = map_state.get_bounds();
    let min = Vec2::from(GameCursor::game_to_world(min.x, min.y, CursorLayer::Room)) - 40.0;
    let max = Vec2::from(GameCursor::game_to_world(max.x, max.y, CursorLayer::Room)) + 40.0;

    rock_map_state.rocks.clear();

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

        transform.translation += rock.movement_speed * time.delta_secs();
        transform.rotate_axis(
            rock.rotation_axis,
            TAU * rock.rotation_speed * time.delta_secs(),
        );

        let pos = GameCursor::world_to_game(
            transform.translation.x,
            transform.translation.y,
            CursorLayer::Room,
        );
        if map_state.room(pos.x, pos.y, MapLayer::Main) {
            map_state.remove(pos.x, pos.y, MapLayer::Main);
            commands.entity(entity).try_despawn_recursive();
        }
        let pos = GameCursor::world_to_game(
            transform.translation.x,
            transform.translation.y,
            CursorLayer::Hook,
        );
        rock_map_state.add(pos.x, pos.y, entity);
    }
}

fn rock_spawner(
    mut commands: Commands,
    map_state: Res<MapState>,
    time: Res<Time>,
    mut last_spawned: Local<f32>,
) {
    if time.elapsed_secs() - *last_spawned < 1.0 {
        return;
    }

    let mut rng = rand::rng();

    let (min, max) = map_state.get_bounds();
    let min = Vec2::from(GameCursor::game_to_world(min.x, min.y, CursorLayer::Room));
    let max = Vec2::from(GameCursor::game_to_world(max.x, max.y, CursorLayer::Room));

    let origin = min - Vec2::new(30.0, 20.0);
    let size = max + Vec2::new(30.0, 20.0) - origin;

    let point = rng.random_range(0.0..size.x * 2.0 + size.y * 2.0);

    let (spawn_point, flight_dir) = if point < size.y {
        let x = 0.0;
        let y = point;
        (Vec2::new(x, y), Vec2::X)
    } else if point < size.y + size.x {
        let x = point - size.y;
        let y = size.y;
        (Vec2::new(x, y), Vec2::NEG_Y)
    } else if point < 2.0 * size.y + size.x {
        let x = size.x;
        let y = 2.0 * size.y + size.x - point;
        (Vec2::new(x, y), Vec2::NEG_X)
    } else {
        let x = 2.0 * size.y + 2.0 * size.x - point;
        let y = 0.0;
        (Vec2::new(x, y), Vec2::Y)
    };

    let spawn_point = origin + spawn_point;
    let flight_dir = Vec2::from_angle(rng.random_range(-FRAC_PI_4..FRAC_PI_4)).rotate(flight_dir);
    let cell_radius = CursorLayer::Room.size() * SQRT_2;

    for IVec2 { x, y } in map_state.primary_blocks() {
        let coords = GameCursor::game_to_world(x, y, CursorLayer::Room);
        let closest_dir = coords - spawn_point;
        let closest_dist = closest_dir.angle_to(flight_dir).sin() * closest_dir.length();
        if closest_dist <= cell_radius {
            return;
        }
    }

    commands.spawn((
        Rock {
            movement_speed: flight_dir.extend(0.0).normalize() * rng.random_range(1.0..3.0),
            rotation_speed: rng.random_range(-1.0..1.0),
            rotation_axis: Transform::from_rotation(Quat::random()).forward(),
        },
        Transform::from_xyz(spawn_point.x, spawn_point.y, 2.0)
            .looking_to(flight_dir.extend(0.0), Vec3::Z)
            .with_scale(Vec3::splat(rng.random_range(0.2..0.5))),
    ));

    *last_spawned = time.elapsed_secs();
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
