use std::marker::PhantomData;

use bevy::{
    asset::RenderAssetUsages, core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    }, image::TextureFormatPixelInfo, prelude::*, render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel},
        render_resource::{
            binding_types::uniform_buffer, encase::internal::WriteInto, BindGroupEntries,
            BindGroupLayout, BindGroupLayoutEntries, CachedPipelineState, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, Extent3d, FragmentState, MultisampleState, Operations,
            Pipeline, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, ShaderType,
            TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
        },
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        MainWorld, RenderApp,
    }
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Default)]
pub struct ProceduralMaterialPlugin<Settings> {
    _pd: PhantomData<Settings>,
}

impl<Settings> Plugin for ProceduralMaterialPlugin<Settings>
where
    Settings: ExtractComponent + ShaderType + WriteInto + Clone + Default + ProceduralMaterial,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<MaterialPrivateTextures>::default(),
            ExtractComponentPlugin::<Settings>::default(),
            UniformComponentPlugin::<Settings>::default(),
        ))
        .add_systems(PostUpdate, init_textures::<Settings>)
        .insert_resource(MaterialSharedTextures::<Settings>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, request_mips::<Settings>)
            .add_render_graph_node::<ProceduralMaterialNode<Settings>>(
                Core3d,
                ProceduralMaterialLabel,
            )
            .add_render_graph_edges(Core3d, (ProceduralMaterialLabel, Node3d::Prepass));
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<ProceduralMaterialPipeline<Settings>>();
    }
}

#[derive(Default, Clone)]
pub enum TextureMode {
    #[default]
    Shared,
    Private,
}

#[derive(Default, Clone)]
pub enum TextureUpdate {
    #[default]
    Once,
    EachFrame,
}

#[derive(EnumIter, Clone, Copy)]
pub enum TextureLayer {
    Diffuse,
    Emissive,
    MetallicRoughness,
    Normal,
}

impl TextureLayer {
    fn texture_format(&self) -> TextureFormat {
        match self {
            TextureLayer::Diffuse => TextureFormat::Rgba8UnormSrgb,
            TextureLayer::Emissive => TextureFormat::Rgba16Float,
            TextureLayer::MetallicRoughness => TextureFormat::Rgba8Unorm,
            TextureLayer::Normal => TextureFormat::Rgba8Unorm,
        }
    }
}

#[derive(Default, Clone)]
struct Textures {
    diffuse: Handle<Image>,
    emissive: Handle<Image>,
    metallic_roughness: Handle<Image>,
    normal: Handle<Image>,
}

impl Textures {
    fn get(&self, layer: TextureLayer) -> Handle<Image> {
        match layer {
            TextureLayer::Diffuse => self.diffuse.clone_weak(),
            TextureLayer::Emissive => self.emissive.clone_weak(),
            TextureLayer::MetallicRoughness => self.metallic_roughness.clone_weak(),
            TextureLayer::Normal => self.normal.clone_weak(),
        }
    }

    fn get_mut(&mut self, layer: TextureLayer) -> &mut Handle<Image> {
        match layer {
            TextureLayer::Diffuse => &mut self.diffuse,
            TextureLayer::Emissive => &mut self.emissive,
            TextureLayer::MetallicRoughness => &mut self.metallic_roughness,
            TextureLayer::Normal => &mut self.normal,
        }
    }
}

#[derive(Component, ExtractComponent, Clone)]
struct MaterialPrivateTextures {
    mips_requested: bool,
    textures: Textures,
}

#[derive(Resource, Default)]
struct MaterialSharedTextures<Settings: Component> {
    mips_requested: bool,
    textures: Textures,
    _pd: PhantomData<Settings>,
}

#[derive(Default)]
pub struct TextureDef {
    pub mode: TextureMode,
    pub update: TextureUpdate,
}

pub trait ProceduralMaterial {
    fn shader() -> &'static str;
    fn size() -> (u32, u32);
    fn texture_def(_: TextureLayer) -> TextureDef {
        TextureDef::default()
    }
}

fn init_textures<Settings: Component + ProceduralMaterial>(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shared_textures: ResMut<MaterialSharedTextures<Settings>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>, Option<&Mesh3d>), Added<Settings>>,
) {
    let size = {
        let (width, height) = Settings::size();
        Extent3d {
            width,
            height,
            ..Default::default()
        }
    };

    let private_image = |format: TextureFormat| {
        println!("gen img");
        let mut img = Image::new_fill(
            size,
            TextureDimension::D2,
            &vec![0; format.pixel_size()],
            format,
            RenderAssetUsages::default(),
        );
        img.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        img
    };

    let mut image = |layer| match Settings::texture_def(layer).mode {
        TextureMode::Private => images.add(private_image(layer.texture_format())),
        TextureMode::Shared => {
            let handle = shared_textures.textures.get_mut(layer);
            if handle.id() == AssetId::default() {
                *handle = images.add(private_image(layer.texture_format()));
            }
            handle.clone()
        }
    };

    for (e, material, mesh) in &query {
        let diffuse = image(TextureLayer::Diffuse);
        let emissive = image(TextureLayer::Emissive);
        let metallic_roughness = image(TextureLayer::MetallicRoughness);
        let normal = image(TextureLayer::Normal);

        let mat = materials.get_mut(material).unwrap();
        // mat.unlit = true;
        mat.base_color = Color::WHITE;
        mat.base_color_texture = Some(diffuse.clone_weak());
        mat.emissive = Color::WHITE.to_linear();
        mat.emissive_texture = Some(emissive.clone_weak());
        mat.perceptual_roughness = 1.0;
        mat.metallic = 1.0;
        mat.metallic_roughness_texture = Some(metallic_roughness.clone_weak());
        mat.normal_map_texture = Some(normal.clone_weak());

        commands.entity(e).insert(MaterialPrivateTextures {
            mips_requested: false,
            textures: Textures {
                diffuse,
                emissive,
                metallic_roughness,
                normal,
            },
        });

        if let Some(mesh) = mesh {
            meshes.get_mut(mesh).unwrap().generate_tangents().unwrap();
        }
    }
}

fn request_mips<Settings: Component + ProceduralMaterial>(
    mut main_world: ResMut<MainWorld>,
    proc_mat_pipeline: Res<ProceduralMaterialPipeline<Settings>>,
    pipeline_cache: Res<PipelineCache>,
) {
    let Some(CachedPipelineState::Ok(Pipeline::RenderPipeline(_))) = pipeline_cache
        .pipelines()
        .nth(proc_mat_pipeline.main_pipeline_id.id())
        .map(|cached| &cached.state)
    else {
        return;
    };

    let mut generate_mips = vec![];

    let mut mat_shared = main_world.resource_mut::<MaterialSharedTextures<Settings>>();
    if !mat_shared.mips_requested {
        generate_mips.extend(TextureLayer::iter().map(|l| (l.clone(), mat_shared.textures.get(l))));
        mat_shared.mips_requested = true;
    }

    main_world
        .query::<&mut MaterialPrivateTextures>()
        .iter_mut(&mut main_world)
        .filter(|t| !t.mips_requested)
        .for_each(|mut mat| {
            generate_mips.extend(TextureLayer::iter().map(|l| (l.clone(), mat.textures.get(l))));
            mat.mips_requested = true;
        });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ProceduralMaterialLabel;

#[derive(Default)]
struct ProceduralMaterialNode<Settings: Default> {
    entities: Vec<Entity>,
    _pd: PhantomData<Settings>,
}

impl<Settings> Node for ProceduralMaterialNode<Settings>
where
    Settings: Component + ShaderType + WriteInto + Default,
{
    fn update(&mut self, world: &mut World) {
        self.entities = world
            .query_filtered::<Entity, With<Settings>>()
            .iter(&world)
            .collect();
    }

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let proc_mat_pipeline = world.resource::<ProceduralMaterialPipeline<Settings>>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let images = world.resource::<RenderAssets<GpuImage>>();
        let Some(main_pipeline) =
            pipeline_cache.get_render_pipeline(proc_mat_pipeline.main_pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<Settings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let main_bind_group = render_context.render_device().create_bind_group(
            "proc_mat_bind_group",
            &proc_mat_pipeline.main_layout,
            &BindGroupEntries::sequential((settings_binding.clone(),)),
        );

        for entity in self.entities.clone() {
            let Some(material) = world.get::<MaterialPrivateTextures>(entity) else {
                continue;
            };
            let Some(index) = world
                .get::<DynamicUniformIndex<Settings>>(entity)
                .map(|i| i.index())
            else {
                continue;
            };

            let gpu_images = TextureLayer::iter()
                .map(|l| images.get(&material.textures.get(l)).unwrap())
                .collect::<Vec<_>>();

            let views = gpu_images
                .iter()
                .map(|gpu_image| {
                    gpu_image.texture.create_view(&TextureViewDescriptor {
                        base_mip_level: 0,
                        mip_level_count: Some(1),
                        ..Default::default()
                    })
                })
                .collect::<Vec<_>>();

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("proc_mat_pass"),
                color_attachments: views
                    .iter()
                    .map(|view| {
                        Some(RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_render_pipeline(main_pipeline);
            render_pass.set_bind_group(0, &main_bind_group, &[index]);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}

#[derive(Resource)]
struct ProceduralMaterialPipeline<Settings> {
    main_layout: BindGroupLayout,
    main_pipeline_id: CachedRenderPipelineId,
    _pd: PhantomData<Settings>,
}

impl<Settings: ShaderType + ProceduralMaterial> FromWorld for ProceduralMaterialPipeline<Settings> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let main_layout = render_device.create_bind_group_layout(
            "proc_mat_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (uniform_buffer::<Settings>(true),),
            ),
        );
        let main_shader = world.load_asset(Settings::shader());
        let main_pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("proc_mat_pipeline".into()),
                    layout: vec![main_layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: main_shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: TextureLayer::iter()
                            .map(|l| {
                                Some(ColorTargetState {
                                    format: l.texture_format(),
                                    blend: None,
                                    write_mask: ColorWrites::ALL,
                                })
                            })
                            .collect(),
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            main_layout,
            main_pipeline_id,
            _pd: PhantomData::default(),
        }
    }
}
