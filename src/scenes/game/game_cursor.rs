use bevy::prelude::*;

use crate::scenes::AppState;

pub struct GameCursorPlugin;

impl Plugin for GameCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_cursor.run_if(
                in_state(AppState::Game)
                    .and(any_with_component::<Window>)
                    .and(any_with_component::<Camera>),
            ),
        );
    }
}

#[derive(Resource)]
pub struct GameCursor {
    pub x: i32,
    pub y: i32,
    pub fx: f32,
    pub fy: f32,
    pub just_pressed: bool,
}

pub enum CursorLayer {
    Room,
    Hook,
}

impl CursorLayer {
    pub fn size(&self) -> f32 {
        match self {
            Self::Room => 2.01,
            Self::Hook => 1.0,
        }
    }
}

impl GameCursor {
    pub fn game_to_world(x: i32, y: i32, layer: CursorLayer) -> Vec2 {
        Vec2::new(x as f32, y as f32) * layer.size()
    }

    pub fn world_to_game(x: f32, y: f32, layer: CursorLayer) -> IVec2 {
        ((Vec2::new(x, y) + 0.5 * layer.size()) / layer.size())
            .floor()
            .as_ivec2()
    }
}

pub fn update_cursor(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    commands.remove_resource::<GameCursor>();

    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.into_inner().cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Z)) else {
        return;
    };

    let Vec3 { x: fx, y: fy, .. } = ray.get_point(distance);
    let IVec2 { x, y } = GameCursor::world_to_game(fx, fy, CursorLayer::Room);
    let just_pressed = mouse.just_pressed(MouseButton::Left);

    commands.insert_resource(GameCursor {
        x,
        y,
        fx,
        fy,
        just_pressed,
    });
}
