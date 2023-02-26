mod grass;
mod water;
mod tree;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::{AssetInspectorPlugin, WorldInspectorPlugin};
use grass::GrassPlugin;
use tree::TreePlugin;

use bevy::{prelude::*, scene::SceneInstance, window::WindowPlugin, pbr::NotShadowReceiver};
use water::{
    setup_reflection_cam, update_reflection_cam, update_reflection_texture, WaterMaterial,
    WaterReflectionTexture,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.5)))
        .insert_resource(AmbientLight {
            color: Color::rgb(1., 0.9, 0.9),
            brightness: 0.5,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "birdylook".to_string(),
                width: 1200.,
                height: 800.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(NoCameraPlayerPlugin)
        // used to show fps in console
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(warblersneeds::warblers_plugin::WarblersPlugin)
        .add_plugin(GrassPlugin)
        .add_plugin(TreePlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(AssetInspectorPlugin::<WaterMaterial>::default())
        .add_plugin(MaterialPlugin::<water::WaterMaterial>::default())
        .add_startup_system(setup)
        .add_startup_system(setup_reflection_cam)
        .add_system(prepare_scene)
        .add_system(update_reflection_cam)
        .add_system(update_reflection_texture)
        .run();
}
/// The [`Player`] component is just a marker for the player entity
#[derive(Component)]
pub struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // spawn player
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-10., 3., 30.),
            ..default()
        })
        .insert(Player)
        .insert(FlyCam)
        .insert(Name::new("PlayerCam"));
    // spawn the scene
    commands.spawn(SceneBundle {
        scene: asset_server.load("scene/game_world.glb#Scene0"),
        ..default()
    });
    
    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

    const HALF_SIZE: f32 = 100.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0,
                far: 1000.0,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-0.5),
            ..default()
        },
        ..default()
    });
}

fn prepare_scene(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Scene>>,
    scene_root_nodes: Query<&Children>,
    objects: Query<(Entity, &Name)>,
    scenes: Query<&Children, With<SceneInstance>>,
    mut water_materials: ResMut<Assets<water::WaterMaterial>>,
    mut leaf_materials: ResMut<Assets<tree::LeafMaterial>>,
    reflection_texture: Res<WaterReflectionTexture>,
    asset_server: Res<AssetServer>,
) {
    for _event in ev_asset.iter() {
        for scene_root in scenes.iter() {
            info!("finished loading scene");
            for &root_node in scene_root.iter() {
                for &scene_objects in scene_root_nodes.get(root_node).unwrap() {
                    if let Ok((e, name)) = objects.get(scene_objects) {
                        if name.contains("Water") {
                            for mesh_entity in scene_root_nodes.get(e).unwrap() {
                                let water_material = water_materials.add(water::WaterMaterial {
                                    base_color: Color::rgba_u8(32, 32, 42, 80),
                                    reflection_image: reflection_texture.texture.clone(),
                                    wave_height: 1.,
                                    direction: Vec2::new(1.5, 0.2),
                                });
                                commands
                                    .entity(*mesh_entity)
                                    .remove::<Handle<StandardMaterial>>();
                                commands.entity(*mesh_entity).insert(water_material);
                            }
                        }
                        if name.contains("Lantern") {
                            let point_light = commands
                                .spawn(PointLightBundle {
                                    point_light: PointLight {
                                        range: 2000.,
                                        intensity: 800.0,
                                        color: Color::rgb(0.9, 0.4, 0.1),
                                        shadows_enabled: true,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .id();
                            commands.entity(e).add_child(point_light);
                        }
                        if name.contains("leaf") {
                            for mesh_entity in scene_root_nodes.get(e).unwrap() {
                                let custom_material = leaf_materials.add(tree::LeafMaterial {
                                    color: Color::YELLOW_GREEN,
                                    color_texture: Some(asset_server.load("shaders/leaves.png")),
                                });
                                commands.entity(*mesh_entity).remove::<Handle<StandardMaterial>>();
                                commands
                                    .entity(*mesh_entity)
                                    .insert((custom_material, NotShadowReceiver));
                            }
                        }
                    }
                }
            }
        }
    }
}
