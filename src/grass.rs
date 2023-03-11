use bevy::{prelude::*, render::primitives::Aabb};
use warbler_grass::prelude::*;

pub struct GrassPlugin;
impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(create_grass)
            .add_plugin(warbler_grass::editor::EditorPlugin);
    }
}
/// Creates the grass in our game world
fn create_grass(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scene_obj: Query<(&Name, &Transform)>,
    mesh_child: Query<(&Aabb, &Handle<Mesh>, &Parent)>,
    mut mesh_assets: EventReader<AssetEvent<Mesh>>,
) {
    // The scene object loads in following relationship in bevy:
    // SceneRoot -> [Ground, Water, Bridge, ..] // Scene objects
    // Ground(Transform) -> GroundMesh // each object has a child with the mesh
    // GroundMesh(Aabb,Handle<Mesh>)

    // wait for the scene to load the ground mesh
    for ev in mesh_assets.iter() {
        if let AssetEvent::Created {
            handle: created_handle,
        } = ev
        {
            // mesh_child is the ground mesh
            for (aabb, handle, parent) in mesh_child.iter() {
                // check if this mesh was created
                if handle != created_handle {
                    continue;
                }
                // get the parent
                let parent_id = parent.get();
                // check if the scene object has the needed components and fetch it accordingly
                let Ok((name, transform)) = scene_obj.get(parent_id) else {
                    continue
                };
                // We gave our ground the name object name "Ground" in blender
                if !name.contains("Ground") {
                    continue;
                }
                // load the height map from image
                let height_map_texture = asset_server.load("grass/height_map.png");
                let height_map = HeightMap {
                    height_map: height_map_texture,
                };
                // load the density map from image
                let density_map_texture = asset_server.load("grass/density_map.png");
                let density_map = DensityMap {
                    density_map: density_map_texture,
                    density: 2.,
                };
                // load the heights of the blades from iamge
                let heights_map_texture = asset_server.load("grass/heights_map.png");
                let warbler_height = WarblerHeight::Texture(heights_map_texture);
                // spawn the grass
                commands
                    .spawn(WarblersBundle {
                        height_map: height_map.clone(),
                        density_map: density_map.clone(),
                        height: warbler_height.clone(),
                        // the aabb of the grass chunk defines the chunk size
                        aabb: aabb.clone(),
                        spatial: SpatialBundle {
                            // we give the chunk the same transform as the ground scene object so their start at the same position
                            // We could also set the grass chunk as child of the ground entity as an alternative
                            transform: transform.clone(),
                            ..default()
                        },
                        ..default()
                    })
                    // we name it so it is easier to identify in the inspector
                    .insert(Name::new("Grass chunk"));
            }
        }
    }
}
