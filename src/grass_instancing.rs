use bevy::{
    core::{Pod, Zeroable},
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{
            lifetimeless::{Read, SQuery, SRes},
            SystemParamItem,
        },
    },
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase, SetItemPipeline,
            TrackedRenderPass,
        },
        render_resource::{
            Buffer, BufferInitDescriptor, BufferUsages, PipelineCache, RenderPipelineDescriptor,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines,
            VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        renderer::RenderDevice,
        view::ExtractedView,
    },
};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GrassInstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub color: [f32; 4],
}
const GRASS_COLOR: [f32; 4] = [0.1, 0.2, 0.01, 1.];
impl GrassInstanceData {
    pub fn new(x: f32, y: f32, z: f32) -> GrassInstanceData {
        GrassInstanceData {
            position: Vec3::new(x, y, z),
            scale: 1.,
            color: GRASS_COLOR,
        }
    }
    pub fn with_scale(mut self, scale: f32) -> GrassInstanceData {
        self.scale = scale;
        self
    }
}

#[derive(Component, Deref)]
pub struct GrassInstanceMaterialData(pub Vec<GrassInstanceData>);

impl ExtractComponent for GrassInstanceMaterialData {
    type Query = &'static GrassInstanceMaterialData;
    type Filter = ();

    fn extract_component(item: QueryItem<'_, Self::Query>) -> GrassInstanceMaterialData {
        GrassInstanceMaterialData(item.0.clone())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<GrassInstanceMaterialData>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

pub fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &GrassInstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/grass_shader.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<GrassInstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 1,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 2,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
    }
}

pub type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

pub struct DrawMeshInstanced;

impl EntityRenderCommand for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<InstanceBuffer>>,
    );

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, instance_buffer_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();
        let instance_buffer = instance_buffer_query.get_inner(item).unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
