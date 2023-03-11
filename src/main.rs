mod grass;
mod scene;
mod water;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::{AssetInspectorPlugin, WorldInspectorPlugin};

use bevy::{prelude::*, window::WindowPlugin};
use grass::GrassPlugin;
use warbler_grass::editor::ray_cast::RayCamera;
use water::{
    setup_reflection_cam, update_reflection_cam, update_reflection_texture, WaterMaterial,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.5)))
        .insert_resource(AmbientLight {
            color: Color::rgb(1., 0.9, 0.9),
            brightness: 0.5,
        })
        // set our window title
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "birdylook".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugin(warbler_grass::warblers_plugin::WarblersPlugin)
        .add_plugin(GrassPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(AssetInspectorPlugin::<WaterMaterial>::default())
        .add_plugin(MaterialPlugin::<water::WaterMaterial>::default())
        .add_startup_system(setup)
        .add_startup_system(setup_reflection_cam)
        .add_system(scene::prepare_scene)
        .add_system(update_reflection_cam)
        .add_system(update_reflection_texture)
        .run();
}
/// The [`Player`] component is just a marker for the player entity
#[derive(Component)]
pub struct Player;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // spawn player with a camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0., 3., 0.),
            ..default()
        })
        // marker component
        .insert(Player)
        // Bundle from the bevy_flycam crate for the camera movement
        .insert(FlyCam)
        // giving it a name makes it easier to identify in the inspector
        .insert(Name::new("Player"))
        // used for the editor in the warbler_grass crate
        .insert(RayCamera::default());

    // spawn the scene
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("scene/game_world.glb#Scene0"),
            ..default()
        })
        .insert(Name::new("Scene root"));
}
