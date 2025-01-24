use bevy::{
    core_pipeline::{
        blit::BLIT_SHADER_HANDLE,
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor, TextureFormatPixelInfo},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel},
        render_resource::{
            binding_types::{sampler, texture_2d},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FilterMode, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            TextureSampleType, TextureViewDescriptor,
        },
        renderer::{RenderContext, RenderDevice},
        sync_world::MainEntity,
        texture::GpuImage,
        MainWorld, RenderApp,
    },
    utils::{hashbrown::HashMap, HashSet},
};

#[derive(Debug, Clone)]
pub enum GenerateMipsMode {
    Once,
    EachFrame,
}

#[derive(Clone, Debug)]
enum GenerateMipsState {
    Spawned,
    Ready,
    Rendered,
}

#[derive(Component, ExtractComponent, Clone, Debug)]
pub struct GenerateMips {
    image: AssetId<Image>,
    mode: GenerateMipsMode,
    state: GenerateMipsState,
}

impl GenerateMips {
    pub fn new(image: AssetId<Image>, mode: GenerateMipsMode) -> Self {
        Self {
            image,
            mode,
            state: GenerateMipsState::Spawned,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct MipmapGeneratorLabel;

#[derive(Default)]
pub struct MipmapGeneratorPlugin;

impl Plugin for MipmapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<GenerateMips>::default())
            .add_systems(
                PostUpdate,
                (cleanup_states, init_mip_textures.after(cleanup_states)),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, readback_states)
            .add_systems(ExtractSchedule, extract_pipelines)
            .add_render_graph_node::<MipmapGeneratorNode>(Core3d, MipmapGeneratorLabel)
            .add_render_graph_edges(Core3d, (Node3d::Prepass, MipmapGeneratorLabel));
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<MipmapGeneratorPipeline>()
            .init_resource::<SpecializedRenderPipelines<MipmapGeneratorPipeline>>();
    }
}

fn cleanup_states(
    mut commands: Commands,
    states: Query<(Entity, &GenerateMips)>,
    images: Res<Assets<Image>>,
) {
    for (entity, state) in states.iter() {
        if !images.contains(state.image) {
            println!("Despawning");
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn init_mip_textures(mut images: ResMut<Assets<Image>>, mut states: Query<&mut GenerateMips>) {
    for mut state in states.iter_mut() {
        if let GenerateMipsState::Spawned = state.state {
            state.state = GenerateMipsState::Ready;
        } else {
            continue;
        }

        let image = images.get_mut(state.image).unwrap();

        let (width, height) = {
            let size = image.size();
            (size.x, size.y)
        };

        let mips = 1 + (width.max(height) as f32).log2() as u32;

        let mut bytes = 0;
        for level in 0..mips {
            bytes += image.texture_descriptor.format.pixel_size()
                * (width >> level) as usize
                * (height >> level) as usize;
        }

        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            ..Default::default()
        });
        image.texture_descriptor.mip_level_count = mips;
        image.data.resize(bytes, 16);
    }
}

fn readback_states(mut world: ResMut<MainWorld>, states: Query<(MainEntity, &GenerateMips)>) {
    for (entity, state) in states.iter() {
        if let Some(mut main_state) = world.get_mut::<GenerateMips>(entity) {
            *main_state = state.clone();
        }
    }
}

#[derive(Component, Clone)]
struct MipmapPipelineId(CachedRenderPipelineId);

fn extract_pipelines(
    mut commands: Commands,
    mut pipelines: ResMut<SpecializedRenderPipelines<MipmapGeneratorPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    mip_gen_pipeline: Res<MipmapGeneratorPipeline>,
    images: Res<RenderAssets<GpuImage>>,
    states: Query<(Entity, &GenerateMips), Without<MipmapPipelineId>>,
) {
    for (entity, state) in states.iter() {
        let image = images.get(state.image).unwrap();
        commands
            .entity(entity)
            .insert(MipmapPipelineId(pipelines.specialize(
                &pipeline_cache,
                &mip_gen_pipeline,
                MipmapGeneratorPipelineKey {
                    texture_format: image.texture_format,
                },
            )));
    }
}

#[derive(Default)]
struct MipmapGeneratorNode {
    images: HashSet<(AssetId<Image>, CachedRenderPipelineId)>,
}

impl Node for MipmapGeneratorNode {
    fn update(&mut self, world: &mut World) {
        let states = world
            .query_filtered::<(Entity, &MipmapPipelineId), With<GenerateMips>>()
            .iter(world)
            .map(|(e, id)| (e, id.clone()))
            .collect::<Vec<_>>();

        let pipeline_cache = world.resource::<PipelineCache>();

        let pipelines = states
            .into_iter()
            .filter_map(|(e, id)| pipeline_cache.get_render_pipeline(id.0).map(|_| (e, id.0)))
            .collect::<HashMap<_, _>>();

        self.images = world
            .query::<(Entity, &mut GenerateMips)>()
            .iter_mut(world)
            .filter_map(|(e, mut s)| {
                let Some(pipeline_id) = pipelines.get(&e) else {
                    return None;
                };
                match (&s.mode, &s.state) {
                    (GenerateMipsMode::Once, GenerateMipsState::Ready)
                    | (GenerateMipsMode::EachFrame, GenerateMipsState::Ready)
                    | (GenerateMipsMode::EachFrame, GenerateMipsState::Rendered) => {
                        s.state = GenerateMipsState::Rendered;
                        Some((s.image, *pipeline_id))
                    }
                    _ => None,
                }
            })
            .collect();
    }

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let mip_gen_pipeline = world.resource::<MipmapGeneratorPipeline>();
        let images = world.resource::<RenderAssets<GpuImage>>();

        for (image, pipeline_id) in &self.images {
            let image = images.get(*image).unwrap();
            let pipeline = pipeline_cache.get_render_pipeline(*pipeline_id).unwrap();

            for mip in 1..image.texture.mip_level_count() {
                let view_prev = image.texture.create_view(&TextureViewDescriptor {
                    base_mip_level: mip - 1,
                    mip_level_count: Some(1),
                    ..Default::default()
                });
                let view_cur = image.texture.create_view(&TextureViewDescriptor {
                    base_mip_level: mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                });
                let bind_group = render_context.render_device().create_bind_group(
                    "mip_gen_bind_group",
                    &mip_gen_pipeline.texture_bind_group,
                    &BindGroupEntries::sequential((&view_prev, &mip_gen_pipeline.sampler)),
                );
                let mut render_pass =
                    render_context.begin_tracked_render_pass(RenderPassDescriptor {
                        label: Some("mip_gen_pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view_cur,
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
            }
        }

        Ok(())
    }
}

#[derive(Resource)]
pub struct MipmapGeneratorPipeline {
    pub texture_bind_group: BindGroupLayout,
    pub sampler: Sampler,
}

impl FromWorld for MipmapGeneratorPipeline {
    fn from_world(render_world: &mut World) -> Self {
        let render_device = render_world.resource::<RenderDevice>();

        let texture_bind_group = render_device.create_bind_group_layout(
            "mip_gen_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture_bind_group,
            sampler,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct MipmapGeneratorPipelineKey {
    pub texture_format: TextureFormat,
}

impl SpecializedRenderPipeline for MipmapGeneratorPipeline {
    type Key = MipmapGeneratorPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("mip_gen_pipeline".into()),
            layout: vec![self.texture_bind_group.clone()],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: BLIT_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fs_main".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: Vec::new(),
            zero_initialize_workgroup_memory: false,
        }
    }
}
