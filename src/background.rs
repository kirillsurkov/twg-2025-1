use bevy::{
    prelude::*,
    render::{
        camera::ScalingMode,
        render_resource::{AsBindGroup, ShaderRef},
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin},
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            .add_systems(Startup, init);
    }
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            hdr: true,
            ..Default::default()
        },
        OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 1.0,
                height: 1.0,
            },
            ..OrthographicProjection::default_2d()
        },
        RenderLayers::layer(1),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(BackgroundMaterial {})),
        RenderLayers::layer(1),
    ));
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BackgroundMaterial {}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "stars.wgsl".into()
    }
}
