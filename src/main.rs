mod water;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::{FRAC_PI_4, PI};
use bevy_flycam::{NoCameraPlayerPlugin, FlyCam};

use bevy::{prelude::*, scene::SceneInstance, window::WindowPlugin};
use water::{WaterReflectionTexture, setup_reflection_cam, update_reflection_cam, update_reflection_texture};

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2.2,
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
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(MaterialPlugin::<water::WaterMaterial>::default())
        .add_startup_system(setup)
        .add_startup_system(setup_reflection_cam)
        .add_system(animate_light_direction)
        .add_system(prepare_scene)
        .add_system(update_reflection_cam)
        .add_system(update_reflection_texture)
        .run();
}
/// The [`Player`] component is just a marker for the player entity
#[derive(Component)]
pub struct Player;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
    // spawn a moving light
    const HALF_SIZE: f32 = 1.0;

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            color: Color::rgb(1.0, 0.8, 0.8),
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
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
                                    base_color: Color::rgba_u8(58, 68, 84, 84),
                                    reflection_image: reflection_texture.texture.clone(),
                                    wave_height: 1.,
                                    direction: Vec2::new(1.,1.),
                                });
                                commands.entity(mesh.clone()).remove::<Handle<StandardMaterial>>();
                                commands.entity(mesh.clone()).insert(water_material);
                            }
                        }
                    }
                }
            }
        }
    }
}

// move the light
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 10.0,
            -FRAC_PI_4,
        );
    }
}

