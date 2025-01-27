use std::{collections::BTreeMap, marker::PhantomData};

use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel},
        render_resource::{
            binding_types::{storage_buffer_read_only, texture_storage_2d_array, uniform_buffer},
            encase::{self, internal::WriteInto},
            AsBindGroup, AsBindGroupError, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
            BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferInitDescriptor,
            BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d,
            FragmentState, IntoBinding, MultisampleState, Operations, OwnedBindingResource,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderRef,
            ShaderSize, ShaderStages, ShaderType, StorageTextureAccess, Texture, TextureDescriptor,
            TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
            TextureViewDescriptor, TextureViewDimension, UniformBuffer, UnpreparedBindGroup,
            VertexState,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        MainWorld, RenderApp,
    },
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount, EnumIter};

#[derive(Default)]
pub struct ProceduralMaterialPlugin<Settings> {
    _pd: PhantomData<Settings>,
}

impl<Settings: ProceduralMaterial> Plugin for ProceduralMaterialPlugin<Settings> {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, ProceduralMaterialExtension>,
        >::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, extract::<Settings>)
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

#[derive(Component, PartialEq, Eq, PartialOrd, Ord)]
struct EntityIndex(u32);

#[derive(EnumIter, EnumCount, Clone, Copy)]
pub enum TextureLayer {
    Diffuse,
    Emissive,
    Metallic,
    Roughness,
    Normal,
}

impl TextureLayer {
    fn texture_format(&self) -> TextureFormat {
        match self {
            TextureLayer::Diffuse => TextureFormat::Rgba8Unorm,
            TextureLayer::Emissive => TextureFormat::Rgba16Float,
            TextureLayer::Metallic => TextureFormat::R8Unorm,
            TextureLayer::Roughness => TextureFormat::R8Unorm,
            TextureLayer::Normal => TextureFormat::Rgba8Unorm,
        }
    }
}

pub trait ProceduralMaterial:
    Component + ShaderType + ShaderSize + WriteInto + Clone + Default
{
    fn shader() -> &'static str;
    fn size() -> (u32, u32);
}

fn extract<Settings: ProceduralMaterial>(
    mut commands: Commands,
    mut main_world: ResMut<MainWorld>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    buffer: Option<Res<ProceduralMaterialBufferRes<Settings>>>,
    mut entity_index: Local<u32>,
    mut invalidated: Local<bool>,
) {
    let added = main_world
        .query_filtered::<(Entity, &Mesh3d), (With<Settings>, Without<EntityIndex>)>()
        .iter(&main_world)
        .map(|(e, m)| (e, m.clone_weak()))
        .collect::<Vec<_>>();

    for (entity, _) in added.iter() {
        main_world
            .entity_mut(*entity)
            .insert(EntityIndex(*entity_index));
        *entity_index += 1;
    }

    let mut meshes = main_world.resource_mut::<Assets<Mesh>>();
    for (_, mesh) in &added {
        meshes.get_mut(mesh).unwrap().generate_tangents().unwrap();
    }

    let sorted = main_world
        .query_filtered::<(Entity, &EntityIndex, &Settings), With<Mesh3d>>()
        .iter(&main_world)
        .map(|(e, i, s)| (i, (e, s.clone())))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect::<Vec<_>>();
    let data = sorted.iter().map(|(_, s)| s.clone()).collect::<Vec<_>>();

    if !added.is_empty() {
        *invalidated = true;
        return;
    }

    if !*invalidated {
        if let Some(buffer) = buffer {
            let mut wrapper = encase::StorageBuffer::<Vec<u8>>::new(Vec::with_capacity(
                data.size().get() as usize,
            ));
            wrapper.write(&data).unwrap();
            render_queue.write_buffer(&buffer.buffer, 0, &wrapper.into_inner());
        }

        return;
    }

    *invalidated = false;

    commands.insert_resource(ProceduralMaterialBufferRes::<Settings> {
        buffer: render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: data.size().get(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        _pd: PhantomData::default(),
    });

    let proc_mat_texture = |layer: TextureLayer| {
        let (width, height) = <Settings as ProceduralMaterial>::size();
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: sorted.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: layer.texture_format(),
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &vec![],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        ProceduralMaterialTexture {
            sampler,
            texture,
            view,
        }
    };

    let textures = ProceduralMaterialTextures {
        color: proc_mat_texture(TextureLayer::Diffuse),
        emissive: proc_mat_texture(TextureLayer::Emissive),
        metallic: proc_mat_texture(TextureLayer::Metallic),
        roughness: proc_mat_texture(TextureLayer::Roughness),
        normal: proc_mat_texture(TextureLayer::Normal),
    };

    commands.insert_resource(ProceduralMaterialTexturesRes::<Settings> {
        textures: textures.clone(),
        _pd: PhantomData::default(),
    });

    let mut materials = main_world
        .resource_mut::<Assets<ExtendedMaterial<StandardMaterial, ProceduralMaterialExtension>>>();

    let materials = sorted
        .iter()
        .enumerate()
        .map(|(i, _)| {
            materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                },
                extension: ProceduralMaterialExtension {
                    textures: textures.clone(),
                    index: i as u32,
                },
            })
        })
        .collect::<Vec<_>>();

    for ((entity, _), material) in sorted.iter().zip(materials) {
        main_world
            .entity_mut(*entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(material));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ProceduralMaterialLabel;

#[derive(Default)]
struct ProceduralMaterialNode<Settings: ProceduralMaterial> {
    _pd: PhantomData<Settings>,
}

impl<Settings: ProceduralMaterial> Node for ProceduralMaterialNode<Settings> {
    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let proc_mat_pipeline = world.resource::<ProceduralMaterialPipeline<Settings>>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(main_pipeline) = pipeline_cache.get_render_pipeline(proc_mat_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let Some(buffer) = world.get_resource::<ProceduralMaterialBufferRes<Settings>>() else {
            return Ok(());
        };

        let Some(textures) = world.get_resource::<ProceduralMaterialTexturesRes<Settings>>() else {
            return Ok(());
        };

        let textures = &textures.textures;

        let main_bind_group = render_context.render_device().create_bind_group(
            "proc_mat_bind_group",
            &proc_mat_pipeline.layout,
            &TextureLayer::iter()
                .enumerate()
                .map(|(i, l)| BindGroupEntry {
                    binding: i as u32,
                    resource: textures.get(l).view.into_binding(),
                })
                .chain([
                    BindGroupEntry {
                        binding: TextureLayer::COUNT as u32,
                        resource: proc_mat_pipeline.globals.binding().unwrap(),
                    },
                    BindGroupEntry {
                        binding: 100,
                        resource: buffer.buffer.as_entire_binding(),
                    },
                ])
                .collect::<Vec<_>>(),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("proc_mat_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &proc_mat_pipeline.dummy_texture_view,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(main_pipeline);
        render_pass.set_bind_group(0, &main_bind_group, &[]);
        render_pass.draw(
            0..3,
            0..textures
                .get(TextureLayer::Diffuse)
                .texture
                .size()
                .depth_or_array_layers,
        );

        Ok(())
    }
}

#[derive(Resource)]
struct ProceduralMaterialPipeline<Settings> {
    layout: BindGroupLayout,
    pipeline_id: CachedRenderPipelineId,
    dummy_texture_view: TextureView,
    globals: UniformBuffer<ProceduralMaterialGlobals>,
    _pd: PhantomData<Settings>,
}

impl<Settings: ProceduralMaterial> FromWorld for ProceduralMaterialPipeline<Settings> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let layout = render_device.create_bind_group_layout(
            "proc_mat_bind_group_layout",
            TextureLayer::iter()
                .enumerate()
                .map(|(i, l)| {
                    texture_storage_2d_array(l.texture_format(), StorageTextureAccess::WriteOnly)
                        .build(i as u32, ShaderStages::FRAGMENT)
                })
                .chain([
                    uniform_buffer::<ProceduralMaterialGlobals>(false)
                        .build(TextureLayer::COUNT as u32, ShaderStages::FRAGMENT),
                    storage_buffer_read_only::<Settings>(false).build(100, ShaderStages::FRAGMENT),
                ])
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let (width, height) = <Settings as ProceduralMaterial>::size();

        let dummy_texture = render_device.create_texture(&{
            let mut desc = Image::default().texture_descriptor;
            desc.size = {
                Extent3d {
                    width,
                    height,
                    ..Default::default()
                }
            };
            desc.usage = TextureUsages::RENDER_ATTACHMENT;
            desc
        });
        let dummy_texture_view = dummy_texture.create_view(&TextureViewDescriptor::default());

        let mut globals = UniformBuffer::from(ProceduralMaterialGlobals {
            texture_size: Vec2::new(width as f32, height as f32),
        });
        globals.write_buffer(render_device, render_queue);

        let shader = world.load_asset(Settings::shader());
        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("proc_mat_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: VertexState {
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: "vertex".into(),
                        buffers: vec![],
                    },
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
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
            dummy_texture_view,
            globals,
            _pd: PhantomData::default(),
        }
    }
}

#[derive(ShaderType, Reflect, Debug, Clone, Default)]
struct ProceduralMaterialGlobals {
    texture_size: Vec2,
}

#[derive(Clone)]
struct ProceduralMaterialTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl ProceduralMaterialTexture {
    fn to_binding(&self, view: u32, sampler: u32) -> [(u32, OwnedBindingResource); 2] {
        [
            (view, OwnedBindingResource::TextureView(self.view.clone())),
            (sampler, OwnedBindingResource::Sampler(self.sampler.clone())),
        ]
    }
}

#[derive(Clone)]
struct ProceduralMaterialTextures {
    color: ProceduralMaterialTexture,
    emissive: ProceduralMaterialTexture,
    metallic: ProceduralMaterialTexture,
    roughness: ProceduralMaterialTexture,
    normal: ProceduralMaterialTexture,
}

impl ProceduralMaterialTextures {
    fn get(&self, layer: TextureLayer) -> &ProceduralMaterialTexture {
        match layer {
            TextureLayer::Diffuse => &self.color,
            TextureLayer::Emissive => &self.emissive,
            TextureLayer::Metallic => &self.metallic,
            TextureLayer::Roughness => &self.roughness,
            TextureLayer::Normal => &self.normal,
        }
    }
}

#[derive(Resource)]
struct ProceduralMaterialBufferRes<Settings: ProceduralMaterial> {
    buffer: Buffer,
    _pd: PhantomData<Settings>,
}

#[derive(Resource)]
struct ProceduralMaterialTexturesRes<Settings: ProceduralMaterial> {
    textures: ProceduralMaterialTextures,
    _pd: PhantomData<Settings>,
}

#[derive(Asset, Clone)]
struct ProceduralMaterialExtension {
    textures: ProceduralMaterialTextures,
    index: u32,
}

impl AsBindGroup for ProceduralMaterialExtension {
    type Data = ();
    type Param = ();

    fn label() -> Option<&'static str> {
        Some("ProceduralMaterialExtension")
    }

    fn bind_group_layout_entries(_: &RenderDevice) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        let binding = 100;
        TextureLayer::iter()
            .enumerate()
            .map(|(i, l)| {
                [
                    BindGroupLayoutEntry {
                        binding: binding + i as u32 * 2,
                        visibility: ShaderStages::all(),
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: binding + i as u32 * 2 + 1,
                        visibility: ShaderStages::all(),
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ]
            })
            .flatten()
            .chain([BindGroupLayoutEntry {
                binding: 150,
                visibility: ShaderStages::all(),
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(<u32 as ShaderType>::min_size()),
                },
                count: None,
            }])
            .collect()
    }

    fn unprepared_bind_group(
        &self,
        _: &BindGroupLayout,
        render_device: &RenderDevice,
        _: &mut bevy::ecs::system::SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        let binding = 100;
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(&self.index).unwrap();
        Ok(UnpreparedBindGroup {
            bindings: TextureLayer::iter()
                .enumerate()
                .map(|(i, l)| {
                    self.textures
                        .get(l)
                        .to_binding(binding + i as u32 * 2, binding + i as u32 * 2 + 1)
                })
                .flatten()
                .chain([(
                    150,
                    OwnedBindingResource::Buffer(render_device.create_buffer_with_data(
                        &BufferInitDescriptor {
                            label: None,
                            contents: buffer.as_ref(),
                            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                        },
                    )),
                )])
                .collect(),
            data: (),
        })
    }
}

impl TypePath for ProceduralMaterialExtension {
    fn type_path() -> &'static str {
        "procedural_material_extension"
    }

    fn short_type_path() -> &'static str {
        "proc_mat_ext"
    }
}

impl MaterialExtension for ProceduralMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "procedural.wgsl".into()
    }
}
