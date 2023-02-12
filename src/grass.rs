use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::{
        primitives::Aabb, mesh::VertexAttributeValues,
    },
    utils::BoxedFuture,
};
use warblersneeds::{prelude::{*, standard_generator::GrassFieldGenerator}, file_loader::GrassFields, generator::GrassGenerator};

pub struct GrassPlugin;
impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(insert_grass_data)
        .add_system(create_grass);
    }
}
const GRASS_PLANE_SIZE: usize = 512;
fn create_grass(
    mut commands: Commands,
    mut res: ResMut<GrassDataRes>,
    asset_loader: ResMut<Assets<GrassFields>>,
    meshes: ResMut<Assets<Mesh>>,
    ground: Query<(&Transform, &Name, &Children)>,
    ground_plane: Query<(&Aabb, &Handle<Mesh>)>,
) {
    if !res.loaded {
        if let Some(grass_data) = asset_loader.get(&res.data) {
            let generator = GrassFieldGenerator { data: grass_data };

            let config = StandardGeneratorConfig {
                density: 0.7,
                height: 5.,
                height_deviation: 0.5,
                seed: Some(0x121),
            };
            let mut grass = generator.generate_grass(config);
            
            for (ground_transform, ground_name, children) in ground.iter() {
                    if ground_name.contains("Ground") {
                        let mut scale_x = 1.;
                        let mut mesh = None;

                        let mut scale_z = 1.;
                        for child in children.iter() {
                            let (aabb, mesh_handle) = ground_plane.get(*child).unwrap();
                            scale_x = aabb.center.x * 2. / GRASS_PLANE_SIZE as f32;
                            scale_z = aabb.center.z * 2. / GRASS_PLANE_SIZE as f32;
                            mesh = meshes.get(mesh_handle);

                        }
                        for blade in grass.0.iter_mut() {
                            blade.position.x *= scale_x;
                            blade.position.z *= scale_z;
                        
                            let position_x = blade.position.x;
                            let position_z = blade.position.z;

                            let mut position_y = 0.;
                            let mut min_distance = 100.;
                            // Really heavy operations here. Could need some real optimization
                            if let Some(VertexAttributeValues::Float32x3(vertex_positions)) =
                                mesh.unwrap().attribute(Mesh::ATTRIBUTE_POSITION)
                            {
                                for position in vertex_positions.iter() {
                                    let distance = (position[0] - position_x).abs()
                                        + (position[2] - position_z).abs();
                                    if distance < min_distance {
                                        position_y = position[1];
                                        if distance < 1. {
                                            break;
                                        }
                                        min_distance = distance;
                                    }
                                }
                            }
                            blade.position.y = position_y;
                        }
                        commands.spawn(WarblersBundle {
                            grass: grass.clone(),
                            transform: ground_transform.clone(),
                            ..default()
                        });
                    }
                    
            }
            
            res.loaded = true;
        }
        
    }
}
pub fn insert_grass_data(mut commands: Commands, server: Res<AssetServer>) {
    let grass_data: Handle<GrassFields> = server.load("layers/grass_placement.ron");
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
    pub data: Handle<GrassFields>,
    pub loaded: bool,
}