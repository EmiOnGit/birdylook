use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::RenderTarget,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
    },
    window::WindowResized,
};

#[derive(Resource)]
pub struct WaterReflectionTexture {
    pub texture: Handle<Image>,
}
use bevy_inspector_egui::prelude::*;

use crate::Player;
#[derive(Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "a21e86a9-84ca-4e8d-8a21-6de9adbaee5e"]
pub struct WaterMaterial {
    #[uniform(0)]
    pub base_color: Color,
    /// Expects values between [0, 1].
    /// A [`wave_height`] of 0 indicates a perfectly quiet surface
    #[uniform(1)]
    #[inspector(min = 0.0, max = 1.0)]
    pub wave_height: f32,
    /// The overall direction of the waves
    #[uniform(2)]
    pub direction: Vec2,

    #[texture(3)]
    #[sampler(4)]
    pub reflection_image: Handle<Image>,
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_shader.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/water_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// When the game window is resized the texture defined by the [`WaterMaterial`] also needs to be adjusted.
pub fn update_reflection_texture(
    mut size_changed: EventReader<WindowResized>,
    mut tex_res: ResMut<WaterReflectionTexture>,
    mut images: ResMut<Assets<Image>>,
    mut water: Query<&mut Handle<WaterMaterial>>,
    mut water_assets: ResMut<Assets<WaterMaterial>>,
) {
    for WindowResized {
        id: _,
        width,
        height,
    } in size_changed.iter()
    {
        let tex = tex_res.as_mut();
        let size = Extent3d {
            width: *width as u32 / 2,
            height: *height as u32 / 2,
            ..default()
        };
        if let Some(img) = images.get_mut(&tex.texture) {
            img.resize(size);
        }
        // If ran in release mode it normally freezes the handle to the texture on resize so
        // we need to reassign the handle even though it should be the same.
        for mat in water.iter_mut() {
            if let Some(mut material) = water_assets.get_mut(&mat) {
                material.reflection_image = tex.texture.clone();
            }
        }
    }
}

/// Creates another camera which renders directly to the texture used for reflection of the water.
/// The camera will be positioned under the player by the [`update_reflection_cam`] system
pub fn setup_reflection_cam(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let size = Extent3d {
        width: window.width() as u32 / 3,
        height: window.height() as u32 / 3,
        ..default()
    };
    // This is the texture that will be rendered to.
    let image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    let image_handle = images.add(image);
    commands.insert_resource(WaterReflectionTexture {
        texture: image_handle.clone(),
    });
    let mut camera = Camera::default();

    camera.target = RenderTarget::Image(image_handle.clone());
    camera.priority = 0;
    commands
        .spawn(Camera3dBundle {
            camera,
            ..default()
        })
        .insert(UiCameraConfig { show_ui: false })
        .insert(Name::new("Reflection Camera"));
}

/// Update the position of the reflection camera according to the current position of the players camera.
pub fn update_reflection_cam(
    mut ref_cam: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_cam: Query<&Transform, With<Player>>,
) {
    if let Ok(player_cam) = player_cam.get_single() {
        if let Ok(mut ref_cam) = ref_cam.get_single_mut() {
            // In a more advanced implementation we would be able to use the y-position of the water surface
            // we'd like to reflect! This assumes that the surface is very close to y = 0 at all times
            ref_cam.translation.x = player_cam.translation.x;
            ref_cam.translation.z = player_cam.translation.z;
            ref_cam.translation.y = -player_cam.translation.y;

            // Quad[-x,y,-z,w] mirrors the Quad at the local z-axis. which is exactly what we want!
            let mut players_rotation = player_cam.rotation.clone();
            players_rotation.x *= -1.;
            players_rotation.z *= -1.;
            ref_cam.rotation = players_rotation;
        }
    }
}
