use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use bevy_shader_utils::ShaderUtilsPlugin;

pub struct TreePlugin;
impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LeafMaterial>::default())
            .add_plugin(ShaderUtilsPlugin);
    }
}


/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for LeafMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/leaf_shader.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/leaf_shader.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct LeafMaterial {
    #[uniform(0)]
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
}


