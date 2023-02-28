use bevy::{
    prelude::*,
    render::{mesh::VertexAttributeValues, primitives::Aabb},
};
use warbler_grass::{prelude::*, grass_spawner::GrassSpawner};
use image;

pub struct GrassPlugin;
impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(create_grass);
    }
}
const GRASS_PLANE_SIZE: usize = 512;
fn create_grass(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    ground: Query<(&Transform, &Name, &Children)>,
    ground_plane: Query<(&Aabb, &Handle<Mesh>)>,
    mut created_successfully: Local<bool>,
) {
    if *created_successfully {
        return;
    }
    let image_bytes = include_bytes!("../assets/layers/unformated/grass_placement.png");
    let image = image::load_from_memory(image_bytes).unwrap().to_rgb32f();
    let image_width = image.width() as usize;
    let mut blade_positions = Vec::new();
    let mut blade_heights = Vec::new();
    for (i, pix )in image.pixels().enumerate() {
        if pix[0] < 0.9 {
            let x = i % image_width;
            let z = i / image_width;
            blade_positions.push(Vec2::new(x as f32,z as f32));
            blade_heights.push(((1. - pix[0]) * 2.).max(0.3))
        }
    }


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
            let mut y_positions = Vec::with_capacity(blade_positions.len());
            for position in blade_positions.iter_mut() {
                position.x *= scale_x;
                position.y *= scale_z;

                let position_x = position.x;
                let position_z = position.y;

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
                y_positions.push(position_y);
            }
            commands.spawn(WarblersBundle {
                grass_spawner: GrassSpawner::new().with_positions_y(y_positions).with_positions_xz(blade_positions.clone()).with_heights(blade_heights.clone()),
                spatial: SpatialBundle {
                    transform: ground_transform.clone(),
                    ..default()
                },
                ..default()
            });
            *created_successfully = true;
        }
    }

}
