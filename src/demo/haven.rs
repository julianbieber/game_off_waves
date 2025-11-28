use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    demo::{terrain::height::TerrainChunk, weapons::WeaponType},
    screens::Screen,
};

pub struct HavenPlugin;

impl Plugin for HavenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_time.run_if(in_state(Screen::Gameplay)))
            .add_plugins(Material2dPlugin::<HavenMaterial>::default());
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct Haven {
    available: fn(u32) -> (WeaponType, u32),
    default_level: u32,
    cost: i32,
}

#[allow(dead_code)]
pub fn setup_havens(_chunk: &TerrainChunk, _commands: &mut Commands) {}

fn update_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<HavenMaterial>>,
    boats: Query<&MeshMaterial2d<HavenMaterial>>,
) {
    for c in boats.iter() {
        if let Some(m) = materials.get_mut(c.0.id()) {
            m.time = Vec4::new(time.elapsed_secs(), 0.0, 0.0, 0.0);
        }
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HavenMaterial {
    #[uniform(0)]
    time: Vec4,
}

const SHADER_PATH: &str = "shaders/haven.wesl";

impl Material2d for HavenMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_PATH.into()
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
