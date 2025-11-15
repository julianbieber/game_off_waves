use avian2d::prelude::{AngularDamping, Collider, CollisionLayers, LinearDamping, Mass, RigidBody};
use bevy::{
    math::ops::atan2,
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    demo::{GameCollisionLayer, Health, movement::MovementController, player::Player},
    screens::Screen,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnerConfig {
            remaining_in_wave: 100,
        })
        .add_systems(Update, update_time.run_if(in_state(Screen::Gameplay)))
        .add_systems(
            Update,
            (eval_spawners, remove_stuck_enemies, enemy_movement)
                .run_if(in_state(Screen::Gameplay)),
        )
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

#[derive(Component)]
struct PositionRecording {
    timer: Timer,
    position: Vec3,
}

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
            if config.remaining_in_wave == 0 {
                break;
            }

            spawner.timer.tick(time.delta());
            let mesh = meshes.add(Rectangle::new(100.0, 100.0));
            let material = materials.add(EnemyMaterial { time: Vec4::ZERO });
            if spawner.timer.is_finished() {
                let collision = CollisionLayers::new(
                    GameCollisionLayer::Enemy,
                    [GameCollisionLayer::Terrain, GameCollisionLayer::Player],
                );
                commands.spawn((
                    Enemy,
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    *transform,
                    Collider::rectangle(100.0, 100.0),
                    RigidBody::Dynamic,
                    MovementController {
                        max_speed: 300.0,
                        ..default()
                    },
                    Mass(10.0),
                    AngularDamping(2.0),
                    LinearDamping(0.2),
                    collision,
                    PositionRecording {
                        timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                        position: transform.translation,
                    },
                    Health(100),
                ));
                config.remaining_in_wave -= 1;
            }
            if spawner.timer.is_finished() {
                spawner.timer.reset();
            }
        }
    }
}

fn remove_stuck_enemies(
    time: Res<Time>,
    mut commands: Commands,
    mut config: ResMut<SpawnerConfig>,
    mut enemies: Query<(Entity, &mut PositionRecording, &Transform), With<Enemy>>,
) {
    for (entity, mut record, transform) in &mut enemies {
        record.timer.tick(time.delta());
        if record.timer.is_finished() {
            record.timer.reset();
            if record.position.distance_squared(transform.translation) < 1000.0 {
                commands.entity(entity).despawn();
                config.remaining_in_wave += 1;
            } else {
                record.position = transform.translation;
            }
        }
    }
}

fn enemy_movement(
    player_position: Query<&Transform, With<Player>>,
    mut enemies: Query<(&mut MovementController, &Transform), (Without<Player>, With<Enemy>)>,
) -> std::result::Result<(), BevyError> {
    let player_position = player_position.single()?.translation;
    for (mut enemy_movement, transform) in &mut enemies {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
        let forward = Vec2::new(angle.cos(), angle.sin());

        let to_player = (player_position - transform.translation).normalize().xy();

        enemy_movement.intent = 10.0;
        let a = to_player;
        let b = forward;
        enemy_movement.rotation_intent = -atan2(a.x * b.y - a.y * b.x, a.x * b.x + a.y * b.y);
        if enemy_movement.rotation_intent.is_nan() {
            enemy_movement.rotation_intent = 0.0;
        }
    }
    Ok(())
}
