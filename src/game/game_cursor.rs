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

    let pos = ray.get_point(distance);
    commands.insert_resource(GameCursor {
        x: ((pos.x + 2.01 * 0.5) / 2.01).floor() as i32,
        y: ((pos.y + 2.01 * 0.5) / 2.01).floor() as i32,
        fx: pos.x,
        fy: pos.y,
        just_pressed: mouse.just_pressed(MouseButton::Left),
    });
}
