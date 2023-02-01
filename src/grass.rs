use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    core_pipeline::core_3d::Transparent3d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin,
        mesh::{Indices, VertexAttributeValues},
        primitives::Aabb,
        render_phase::AddRenderCommand,
        render_resource::{PrimitiveTopology, SpecializedMeshPipelines},
        view::NoFrustumCulling,
        RenderApp, RenderStage,
    },
    utils::BoxedFuture,
};
use rand::Rng;

use crate::grass_instancing::{
    prepare_instance_buffers, queue_custom, CustomPipeline, DrawCustom, GrassInstanceData,
    GrassInstanceMaterialData,
};
pub struct GrassPlugin;
impl Plugin for GrassPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<GrassDataAsset>()
            .init_asset_loader::<GrassDataAssetLoader>()
            .add_startup_system(insert_grass_data)
            .add_system(create_grass);
        app.add_plugin(ExtractComponentPlugin::<GrassInstanceMaterialData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CustomPipeline>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}
const GRASS_PLANE_SIZE: usize = 512;
fn create_grass(
    mut commands: Commands,
    mut res: ResMut<GrassDataRes>,
    asset_loader: ResMut<Assets<GrassDataAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    ground: Query<(&Transform, &Name, &Children)>,
    ground_plane: Query<(&Aabb, &Handle<Mesh>)>,
) {
    if !res.loaded {
        let mut rng = rand::thread_rng();
        let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        grass_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0., 0., 0.], [0.5, 0., 0.], [0., 0., 0.5], [0.25, 2., 0.25]],
        );
        grass_mesh.set_indices(Some(Indices::U32(vec![1, 0, 3, 2, 1, 3, 0, 2, 3])));
        let grass_mesh_handle = meshes.add(grass_mesh);
        for (ground_transform, ground_name, children) in ground.iter() {
            if ground_name.contains("Ground") {
                if let Some(data) = asset_loader.get(&res.data) {
                    let mut instance_vector: Vec<GrassInstanceData> = Vec::with_capacity(60_000);

                    let mut scale_x = ground_transform.scale.x;
                    let mut scale_z = ground_transform.scale.z;
                    let mut mesh = None;
                    let mut count = 0;

                    for child in children.iter() {
                        let (aabb, mesh_handle) = ground_plane.get(*child).unwrap();
                        scale_x = aabb.center.x * 2. / GRASS_PLANE_SIZE as f32;
                        scale_z = aabb.center.z * 2. / GRASS_PLANE_SIZE as f32;
                        mesh = meshes.get(mesh_handle);
                    }
                    for value in data.0.iter() {
                        let rect: GrassRect = value.into();
                        if rect.id != 0 {
                            let start_x = ground_transform.translation.x;
                            let start_z = ground_transform.translation.z;
                            let start_y = ground_transform.translation.y;
                            let position_x = rect.x as f32 * scale_x;
                            let center_x = position_x + rect.w as f32 / 2. * scale_x;
                            let position_z = rect.z as f32 * scale_z;
                            let center_z = position_z + rect.h as f32 / 2. * scale_z;

                            let mut position_y = 0.;
                            let mut min_distance = 100.;
                            // Really heavy operations here. Could need some real optimization
                            if let Some(VertexAttributeValues::Float32x3(vertex_positions)) =
                                mesh.unwrap().attribute(Mesh::ATTRIBUTE_POSITION)
                            {
                                for position in vertex_positions.iter() {
                                    let distance = (position[0] - center_x).abs()
                                        + (position[2] - center_z).abs();
                                    if distance < min_distance {
                                        position_y = position[1];
                                        if distance < 1. {
                                            break;
                                        }
                                        min_distance = distance;
                                    }
                                }
                            }
                            for _ in 0..(rect.w * rect.h / 4) {
                                let (x, scale, z): (f32, f32, f32) = rng.gen();
                                let real_x = position_x + start_x + (x * rect.w as f32) * scale_x;
                                let real_z = position_z + start_z + (z * rect.h as f32) * scale_z;
                                let instance =
                                    GrassInstanceData::new(real_x, position_y + start_y, real_z)
                                        .with_scale(scale / 2. + 0.2);
                                instance_vector.push(instance);
                                count += 1;
                            }
                        }
                    }
                    let grass_blade_info = format!("loaded {} grass blades", count);
                    info!(grass_blade_info);

                    commands.spawn((
                        grass_mesh_handle.clone(),
                        GrassInstanceMaterialData(instance_vector),
                        SpatialBundle::VISIBLE_IDENTITY,
                        NoFrustumCulling,
                    ));
                    res.loaded = true;
                }
            }
        }
    }
}
fn insert_grass_data(mut commands: Commands, server: Res<AssetServer>) {
    let grass_data: Handle<GrassDataAsset> = server.load("layers/grass_placement.ron");
    let grass_res = GrassDataRes {
        data: grass_data,
        loaded: false,
    };
    commands.insert_resource(grass_res);
}

/// The asset is in the format of the [GrassRect]
/// and can be converted at any time!
#[derive(Debug, TypeUuid, serde::Deserialize, Reflect)]
#[uuid = "39a3dc56-aa9c-4543-8640-a018b74b5052"]
pub struct GrassDataAsset(pub Vec<[u16; 5]>);
#[derive(Debug, serde::Deserialize, Reflect, FromReflect)]
pub struct GrassRect {
    pub id: u16,
    pub x: u16,
    pub z: u16,
    pub w: u16,
    pub h: u16,
}
impl From<&[u16; 5]> for GrassRect {
    fn from(value: &[u16; 5]) -> Self {
        GrassRect {
            id: value[0],
            x: value[1],
            z: value[2],
            w: value[3],
            h: value[4],
        }
    }
}
#[derive(Resource)]
pub struct GrassDataRes {
    pub data: Handle<GrassDataAsset>,
    pub loaded: bool,
}
#[derive(Default)]
pub struct GrassDataAssetLoader;
impl AssetLoader for GrassDataAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let custom_asset = ron::de::from_bytes::<GrassDataAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }
    /// in practice almost all files should be able to work here
    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
