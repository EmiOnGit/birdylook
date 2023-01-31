mod water;

use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::{FRAC_PI_4, PI};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::{prelude::*, scene::SceneInstance, window::WindowPlugin};
use water::{
    setup_reflection_cam, update_reflection_cam, update_reflection_texture, WaterReflectionTexture,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.4)))

        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
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
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin)
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
            transform: Transform::from_xyz(0., 3., 0.),
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
    
}

fn prepare_scene(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Scene>>,
    scene_root_nodes: Query<&Children>,
    objects: Query<(Entity, &Name)>,
    scenes: Query<&Children, With<SceneInstance>>,
    mut water_materials: ResMut<Assets<water::WaterMaterial>>,
    reflection_texture: Res<WaterReflectionTexture>,
) {
    for _event in ev_asset.iter() {
        for scene_root in scenes.iter() {
            info!("finished loading scene");
            for &root_node in scene_root.iter() {
                for &scene_objects in scene_root_nodes.get(root_node).unwrap() {
                    if let Ok((e, name)) = objects.get(scene_objects) {
                        if name.contains("Water") {
                            for mesh in scene_root_nodes.get(e).unwrap() {
                                let water_material = water_materials.add(water::WaterMaterial {
                                    base_color: Color::rgba_u8(28, 28, 44, 84),
                                    reflection_image: reflection_texture.texture.clone(),
                                    wave_height: 1.,
                                    direction: Vec2::new(1., 1.),
                                });
                                commands
                                    .entity(mesh.clone())
                                    .remove::<Handle<StandardMaterial>>();
                                commands.entity(mesh.clone()).insert(water_material);
                            }
                        }
                        if name.contains("Lantern"){
                            let point_light = commands
                                .spawn(PointLightBundle {
                                    point_light: PointLight {
                                        range: 3000.,
                                        intensity: 1600.0, 
                                        color: Color::rgb(0.9,0.5,0.2),
                                        shadows_enabled: true,
                                        ..default()
                                    },
                                    ..default()
                                }).id();
                            commands.entity(e).add_child(point_light);

                        }
                    }
                }
            }
        }
    }
}
