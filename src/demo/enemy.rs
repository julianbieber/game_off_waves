use avian2d::prelude::{
    AngularDamping, Collider, CollisionLayers, CollisionStart, LinearDamping, Mass, RigidBody,
};
use bevy::{
    math::ops::atan2,
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    PausableSystems,
    demo::{
        GameCollisionLayer, Health,
        movement::MovementController,
        player::{EnemiesKilled, Player, PlayerHealth},
        upgrade::AvailableUpgrades,
    },
    menus::Menu,
    screens::Screen,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnerConfig {
            time_to_spawn: Timer::from_seconds(10.0, TimerMode::Repeating),
            per_wave: 10,
            one_time: 0,
        })
        .add_systems(
            Update,
            update_time
                .in_set(PausableSystems)
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(
            Update,
            (
                eval_spawners,
                remove_stuck_enemies,
                enemy_movement,
                despawn_dead,
                enemies_hit_player,
            )
                .in_set(PausableSystems)
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_plugins(Material2dPlugin::<EnemyMaterial>::default());
    }
}

fn despawn_dead(
    enemies: Query<(Entity, &Health)>,
    mut commands: Commands,
    mut killed: ResMut<EnemiesKilled>,
    mut shop: ResMut<AvailableUpgrades>,
) {
    for (e, h) in enemies {
        if h.0 <= 0 {
            commands.entity(e).despawn();
            killed.amount += 1;
            shop.gold += 1;
        }
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
pub struct Spawner {}

#[derive(Resource)]
pub struct SpawnerConfig {
    pub time_to_spawn: Timer,
    pub per_wave: usize,
    pub one_time: usize,
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
    player_position: Query<&Transform, With<Player>>,
) -> Result<(), BevyError> {
    let player_position = player_position.single()?;
    config.time_to_spawn.tick(time.delta());
    let min_dist = 500.0 * 500.0;
    let max_dist = 1000.0 * 1000.0;
    if config.time_to_spawn.is_finished() {
        let mut spawn_counter = 0;
        for (_spawner, transform) in &mut spawners {
            let dist = transform
                .translation
                .distance_squared(player_position.translation);
            if dist > min_dist && dist < max_dist && spawn_counter < config.per_wave {
                commands.spawn(enemy_bundle(transform, &mut meshes, &mut materials));
                spawn_counter += 1;
            }
        }
        config.one_time += config.per_wave - spawn_counter;
    } else {
        for (_spawner, transform) in &mut spawners {
            let dist = transform
                .translation
                .distance_squared(player_position.translation);
            if dist > min_dist && dist < max_dist && config.one_time > 0 {
                commands.spawn(enemy_bundle(transform, &mut meshes, &mut materials));
                config.one_time -= 1;
            }
        }
    }
    Ok(())
}

fn enemy_bundle(
    transform: &Transform,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<EnemyMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Rectangle::new(100.0, 100.0));
    let material = materials.add(EnemyMaterial { time: Vec4::ZERO });
    let collision = CollisionLayers::new(
        GameCollisionLayer::Enemy,
        [GameCollisionLayer::Terrain, GameCollisionLayer::Player],
    );
    (
        Enemy,
        DespawnOnEnter(Menu::Shop),
        DespawnOnExit(Screen::Gameplay),
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
    )
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
                config.one_time += 1;
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

fn enemies_hit_player(
    mut collisions: MessageReader<CollisionStart>,
    enemies: Query<Entity, (With<Enemy>, Without<Player>)>,
    mut player: Query<&mut PlayerHealth, Without<Enemy>>,
    mut commands: Commands,
) {
    for collision in collisions.read() {
        let _enemy = {
            if let Some(enemy) = [collision.body1, collision.body2]
                .iter()
                .flatten()
                .find(|e| enemies.contains(**e))
            {
                *enemy
            } else {
                // No enemy part of the collision
                break;
            }
        };

        let player_entity = {
            if let Some(player) = [collision.body1, collision.body2]
                .iter()
                .flatten()
                .find(|e| player.contains(**e))
            {
                *player
            } else {
                // No player part of the collision
                break;
            }
        };

        // unwrap is safe due to the previous check
        let mut player = player.get_mut(player_entity).unwrap();
        player.current -= 1;

        dbg!(player.current);
        commands.entity(_enemy).despawn();
    }
}
