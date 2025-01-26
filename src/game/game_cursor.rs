use bevy::prelude::*;

pub struct GameCursorPlugin;

impl Plugin for GameCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cursor);
    }
}

#[derive(Resource)]
pub struct GameCursor {
    pub x: i32,
    pub y: i32,
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

    let pos = ((ray.get_point(distance) + 2.01 * 0.5) / 2.01).floor().xy();
    commands.insert_resource(GameCursor {
        x: pos.x as i32,
        y: pos.y as i32,
        just_pressed: mouse.just_pressed(MouseButton::Left),
    });
}
