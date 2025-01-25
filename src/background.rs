use bevy::{
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::uniform_buffer, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            Extent3d, FragmentState, MultisampleState, Operations, PipelineCache, PrimitiveState,
            RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
            ShaderStages, ShaderType, TextureFormat, UniformBuffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::ViewTarget,
        RenderApp,
    },
};

#[derive(Resource, Reflect, Clone)]
struct BackgroundPluginSettings {
    shader: String,
}

pub struct BackgroundPlugin {
    settings: BackgroundPluginSettings,
}

impl BackgroundPlugin {
    pub fn new<T: AsRef<str>>(shader: T) -> Self {
        Self {
            settings: BackgroundPluginSettings {
                shader: shader.as_ref().to_string(),
            },
        }
    }
}

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<RenderBackground>::default())
            .register_type::<BackgroundPluginSettings>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .insert_resource(self.settings.clone())
            .add_render_graph_node::<ViewNodeRunner<BackgroundNode>>(Core3d, BackgroundLabel)
            .add_render_graph_edges(
                Core3d,
                (Node3d::Prepass, BackgroundLabel, Node3d::StartMainPass),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<BackgroundPipeline>();
    }
}

#[derive(Component, ExtractComponent, Default, Clone, Copy)]
pub struct RenderBackground;

#[derive(ShaderType)]
struct BackgroundGlobals {
    time: f32,
    texture_size: Vec2,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct BackgroundLabel;

#[derive(Default)]
struct BackgroundNode;

impl ViewNode for BackgroundNode {
    type ViewQuery = (&'static ViewTarget, &'static RenderBackground);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let render_queue = world.resource::<RenderQueue>();
        let post_process_pipeline = world.resource::<BackgroundPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let mut globals = {
            let time = world.resource::<Time>().elapsed_secs();
            let Extent3d {
                width,
                height,
                depth_or_array_layers: _,
            } = view_target.main_texture().size();
            UniformBuffer::from(BackgroundGlobals {
                time,
                texture_size: Vec2::new(width as f32, height as f32),
            })
        };

        globals.write_buffer(render_context.render_device(), render_queue);

        let bind_group = render_context.render_device().create_bind_group(
            "background_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::single(&globals),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("background_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view_target.main_texture_view(),
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct BackgroundPipeline {
    layout: BindGroupLayout,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for BackgroundPipeline {
    fn from_world(world: &mut World) -> Self {
        let settings = world.resource::<BackgroundPluginSettings>();

        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            "background_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (uniform_buffer::<BackgroundGlobals>(false),),
            ),
        );

        let shader = world.load_asset(&settings.shader);

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("background_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::Rgba16Float,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            pipeline_id,
        }
    }
}
