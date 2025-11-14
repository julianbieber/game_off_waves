use avian2d::prelude::{Collider, RigidBody};
use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::screens::Screen;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnerConfig {
            remaining_in_wave: 100,
        })
        .add_systems(Update, update_time.run_if(in_state(Screen::Gameplay)))
        .add_systems(Update, eval_spawners.run_if(in_state(Screen::Gameplay)))
        .add_plugins(Material2dPlugin::<EnemyMaterial>::default());
    }
}
fn update_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<EnemyMaterial>>,
    boats: Query<&MeshMaterial2d<EnemyMaterial>>,
) {
    for c in boats.iter() {
        if let Some(m) = materials.get_mut(c.0.id()) {
            m.time = Vec4::new(time.elapsed_secs(), 0.0, 0.0, 0.0);
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct EnemyMaterial {
    #[uniform(0)]
    time: Vec4,
}

const ENEMY_SHADER_PATH: &str = "shaders/enemy.wesl";

impl Material2d for EnemyMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        ENEMY_SHADER_PATH.into()
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}

#[derive(Component)]
pub struct Spawner {
    pub timer: Timer,
}

#[derive(Resource)]
pub struct SpawnerConfig {
    pub remaining_in_wave: u32,
}

#[derive(Component)]
pub struct Enemy;

fn eval_spawners(
    time: Res<Time>,
    mut commands: Commands,
    mut config: ResMut<SpawnerConfig>,
    mut spawners: Query<(&mut Spawner, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<EnemyMaterial>>,
) {
    if config.remaining_in_wave > 0 {
        for (mut spawner, transform) in &mut spawners {
            spawner.timer.tick(time.delta());
            let mesh = meshes.add(Rectangle::new(100.0, 100.0));
            let material = materials.add(EnemyMaterial { time: Vec4::ZERO });
            if spawner.timer.is_finished() && config.remaining_in_wave > 0 {
                commands.spawn((
                    Enemy,
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    transform.clone(),
                    Collider::rectangle(100.0, 100.0),
                    RigidBody::Dynamic,
                ));
                config.remaining_in_wave -= 1;
            }
        }
    }
}
