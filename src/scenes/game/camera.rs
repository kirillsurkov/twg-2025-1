use bevy::{
    core_pipeline::{
        bloom::Bloom, oit::OrderIndependentTransparencySettings, tonemapping::DebandDither,
    },
    input::mouse::MouseWheel,
    prelude::*,
};

use crate::{
    components::background::{BackgroundPluginSettings, RenderBackground},
    scenes::{AppSceneRoot, AppState},
};

pub struct GameCameraPlugin(pub Vec3);

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup)
            .add_systems(
                Update,
                camera_control.run_if(in_state(AppState::Game).and(any_with_component::<Camera>)),
            )
            .insert_resource(TargetPos(self.0));
    }
}

#[derive(Resource, Deref, DerefMut)]
struct TargetPos(Vec3);

fn setup(mut commands: Commands, root_entity: Res<AppSceneRoot>, target_pos: Res<TargetPos>) {
    commands.insert_resource(BackgroundPluginSettings {
        shader: "stars.wgsl".to_string(),
    });

    commands.entity(root_entity.world).with_child((
        Camera3d::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::None,
            ..Default::default()
        },
        RenderBackground,
        Msaa::Off,
        OrderIndependentTransparencySettings {
            layer_count: 32,
            alpha_threshold: 0.01,
        },
        Bloom::NATURAL,
        DebandDither::Enabled,
        Transform::from_translation(target_pos.0),
    ));
}

fn camera_control(
    mut wheel: EventReader<MouseWheel>,
    mut target_pos: ResMut<TargetPos>,
    mut camera: Query<Option<&mut Transform>, With<Camera3d>>,
    time: Res<Time>,
) {
    let Ok(Some(mut camera)) = camera.get_single_mut() else {
        return;
    };

    for event in wheel.read() {
        **target_pos = **target_pos - Vec3::new(0.0, 0.0, event.y);
    }

    target_pos.z = target_pos.z.max(2.5).min(40.0);

    let diff = **target_pos - camera.translation;
    camera.translation += diff * time.delta_secs() / 0.1;
}
