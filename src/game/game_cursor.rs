use bevy::prelude::*;

pub struct GameCursorPlugin;

impl Plugin for GameCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cursor.run_if(any_with_component::<Window>));
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

impl GameCursor {
    pub const CELL_SIZE: f32 = 2.00;
    const GAP: f32 = 0.01;

    pub fn game_to_world(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32, y as f32) * (Self::CELL_SIZE + Self::GAP)
    }

    pub fn world_to_game(x: f32, y: f32) -> IVec2 {
        ((Vec2::new(x, y) + (Self::CELL_SIZE + Self::GAP) * 0.5) / (Self::CELL_SIZE + Self::GAP))
            .floor()
            .as_ivec2()
    }
}

fn update_cursor(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    commands.remove_resource::<GameCursor>();

    let (camera, camera_transform) = camera.into_inner();
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
    let IVec2 { x, y } = GameCursor::world_to_game(fx, fy);
    let just_pressed = mouse.just_pressed(MouseButton::Left);

    commands.insert_resource(GameCursor {
        x,
        y,
        fx,
        fy,
        just_pressed,
    });
}
