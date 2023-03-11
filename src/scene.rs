//! Used for handling and loading the scene
use crate::water::{self, WaterMaterial};
use bevy::prelude::*;
use bevy::scene::SceneInstance;

// prepares the loaded scene
// This mostly consists of adding all components/ materials to objects of the scene
pub fn prepare_scene(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Scene>>,
    scene_root_nodes: Query<&Children>,
    objects: Query<(Entity, &Name)>,
    scenes: Query<&Children, With<SceneInstance>>,
    mut water_materials: ResMut<Assets<water::WaterMaterial>>,
    reflection_texture: Res<water::WaterReflectionTexture>,
) {
    for _event in ev_asset.iter() {
        for scene_root in scenes.iter() {
            info!("finished loading scene");
            for &root_node in scene_root.iter() {
                for &scene_objects in scene_root_nodes.get(root_node).unwrap() {
                    let Ok((e, name)) = objects.get(scene_objects) else {
                        continue;
                    };
                    if name.contains("Water") {
                        for mesh_entity in scene_root_nodes.get(e).unwrap() {
                            // we always need to remove the standard material, added by default, first
                            commands
                                .entity(*mesh_entity)
                                .remove::<Handle<StandardMaterial>>();
                            // create our standard water material
                            let water_material = water_materials
                                .add(water_material(reflection_texture.texture.clone()));
                            // add it to the entity
                            commands.entity(*mesh_entity).insert(water_material);
                        }
                    }
                    if name.contains("Lantern") {
                        // create a entity with the light
                        let point_light = commands.spawn(lantern_point_light()).id();
                        // add it to the lantern
                        commands.entity(e).add_child(point_light);
                    }
                }
            }
        }
    }
}
// loads the default water material used
fn water_material(reflection_texture: Handle<Image>) -> WaterMaterial {
    water::WaterMaterial {
        base_color: Color::rgba_u8(32, 32, 42, 80),
        reflection_image: reflection_texture,
        wave_height: 1.,
        direction: Vec2::new(1.5, 0.2),
    }
}
// loads the default point light used for lanterns
fn lantern_point_light() -> PointLightBundle {
    PointLightBundle {
        point_light: PointLight {
            range: 2000.,
            intensity: 800.0,
            color: Color::rgb(0.9, 0.4, 0.1),
            shadows_enabled: true,
            ..default()
        },
        ..default()
    }
}
