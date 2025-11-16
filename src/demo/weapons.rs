use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    demo::{Health, enemy::Enemy, forward_vec, player::PlayerStats},
    screens::Screen,
};

pub struct WeaponPlugin;

#[derive(Clone)]
#[allow(dead_code)]
pub enum WeaponType {
    Canon {
        cooldown: Timer,
        damage: f32,
    },
    FireMage {
        cooldown: Timer,
        direct_damage: f32,
        burning_stacks: i32,
        radius: f32,
        range: f32,
    },
}

impl WeaponType {
    pub fn default_cannon(player: &PlayerStats) -> WeaponType {
        WeaponType::Canon {
            cooldown: Timer::from_seconds(
                3.0 * player.projectile_rate_percentage,
                TimerMode::Repeating,
            ),
            damage: 30.0,
        }
    }

    pub fn default_fire_mage(player: &PlayerStats) -> WeaponType {
        WeaponType::FireMage {
            cooldown: Timer::from_seconds(
                0.1 * player.projectile_rate_percentage,
                TimerMode::Repeating,
            ),
            direct_damage: 10.0,
            burning_stacks: 0,
            radius: 300.0,
            range: 200.0,
        }
    }

    /// This function does targetting and timer eval
    /// therefore it returns the position (and orientation of where it appears)
    /// The user transform tells us, where the user is positiond and oriented
    fn eval(
        &mut self,
        commands: &mut Commands,
        time: &Time,
        user_transform: Transform,
        enemies: &Query<&Transform, With<Enemy>>,
        player: &PlayerStats,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<WeaponMaterial>,
    ) {
        match self {
            WeaponType::Canon { cooldown, damage } => {
                cooldown.tick(time.delta());
                if cooldown.is_finished() {
                    let mesh = meshes.add(Rectangle::new(30.0, 30.0));
                    let material = materials.add(WeaponMaterial { time: Vec4::ZERO });
                    let b = (
                        CanonBall {
                            speed: 500.0 * player.projectile_speed_percentage,
                            damage: *damage * player.projectile_damage_percentage,
                        },
                        user_transform,
                        Mesh2d(mesh),
                        MeshMaterial2d(material),
                        DespawnAfter(Timer::from_seconds(3.0, TimerMode::Once)),
                    );
                    commands.spawn(b);
                }
            }
            WeaponType::FireMage {
                cooldown,
                range,
                direct_damage,
                burning_stacks,
                radius,
            } => {
                cooldown.tick(time.delta());
                if cooldown.is_finished() {
                    let radius_sq = *range * *range;
                    if let Some(target) = enemies.iter().find(|e| {
                        user_transform.translation.distance_squared(e.translation) < radius_sq
                    }) {
                        let mesh = meshes.add(Rectangle::new(*radius * 2.0, *radius * 2.0));
                        let material = materials.add(WeaponMaterial {
                            time: Vec4::new(0.0, 1.0, 0.0, 0.0),
                        });
                        let b = (
                            FlameStrike {
                                damage: *direct_damage * player.explosion_damage_percentage,
                                _burning_stacks: *burning_stacks,
                                radius: *radius,
                            },
                            *target,
                            Mesh2d(mesh),
                            MeshMaterial2d(material),
                            DespawnAfter(Timer::from_seconds(1.0, TimerMode::Once)),
                        );
                        commands.spawn(b);
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CanonBall {
    pub speed: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct FlameStrike {
    pub damage: f32,
    pub _burning_stacks: i32,
    pub radius: f32,
}

#[derive(Component)]
pub struct DespawnAfter(Timer);

#[derive(Component)]
pub struct WeaponSlots {
    pub left: [Option<WeaponType>; 3],
    pub right: [Option<WeaponType>; 3],
    pub front: Option<WeaponType>,
}

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                cannonball_flight,
                update_time,
                eval_weapons,
                despawn_after_x,
                cannon_ball_hit,
            )
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_observer(flamestrike_hits)
        .add_plugins(Material2dPlugin::<WeaponMaterial>::default());
    }
}

fn eval_weapons(
    time: Res<Time>,
    mut weapon_holders: Query<(&mut WeaponSlots, &Transform, &PlayerStats)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WeaponMaterial>>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for (mut weapon_holder, transform, player) in &mut weapon_holders {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
        let forward = Vec2::new(angle.cos(), angle.sin());

        for (i, left_slot) in weapon_holder.left.iter_mut().enumerate() {
            if let Some(left_slot) = left_slot {
                let weapon_transform = left_weapon_transform(transform, forward, angle, i);
                left_slot.eval(
                    &mut commands,
                    &time,
                    weapon_transform,
                    &enemies,
                    player,
                    &mut meshes,
                    &mut materials,
                );
            }
        }
        for (i, right_slot) in weapon_holder.right.iter_mut().enumerate() {
            if let Some(right_slot) = right_slot {
                let weapon_transform = right_weapon_transform(transform, forward, angle, i);
                right_slot.eval(
                    &mut commands,
                    &time,
                    weapon_transform,
                    &enemies,
                    player,
                    &mut meshes,
                    &mut materials,
                );
            }
        }

        if let Some(front) = &mut weapon_holder.front {
            let weapon_position = transform.translation.xy() + forward * 100.0;
            let weapon_transform =
                Transform::from_translation(Vec3::new(weapon_position.x, weapon_position.y, 0.0))
                    .with_rotation(Quat::from_axis_angle(
                        Vec3::Z,
                        angle - std::f32::consts::FRAC_PI_2,
                    ));

            front.eval(
                &mut commands,
                &time,
                weapon_transform,
                &enemies,
                player,
                &mut meshes,
                &mut materials,
            );
        }
    }
}

const SIDE_OFFSET: f32 = 30.0;
const BETWEEN_SIDE: f32 = 30.0;
fn left_weapon_transform(
    transform: &Transform,
    forward: Vec2,
    forward_angle: f32,
    i: usize,
) -> Transform {
    let i = i as f32 - 1.0;

    let left = Vec2::new(
        (forward_angle - std::f32::consts::FRAC_PI_2).cos(),
        (forward_angle - std::f32::consts::FRAC_PI_2).sin(),
    );
    let weapon_position =
        transform.translation.xy() + left * SIDE_OFFSET + i * forward * BETWEEN_SIDE;
    Transform::from_translation(Vec3::new(weapon_position.x, weapon_position.y, 0.0)).with_rotation(
        Quat::from_axis_angle(Vec3::Z, forward_angle + std::f32::consts::PI),
    )
}

fn right_weapon_transform(
    transform: &Transform,
    forward: Vec2,
    forward_angle: f32,
    i: usize,
) -> Transform {
    let i = i as f32 - 1.0;

    let right = Vec2::new(
        (forward_angle + std::f32::consts::FRAC_PI_2).cos(),
        (forward_angle + std::f32::consts::FRAC_PI_2).sin(),
    );
    let weapon_position =
        transform.translation.xy() + right * SIDE_OFFSET + i * forward * BETWEEN_SIDE;
    Transform::from_translation(Vec3::new(weapon_position.x, weapon_position.y, 0.0))
        .with_rotation(Quat::from_axis_angle(Vec3::Z, forward_angle))
}

fn cannonball_flight(mut balls: Query<(&mut Transform, &CanonBall)>, time: Res<Time>) {
    for (mut ball, stats) in &mut balls {
        let forward = forward_vec(&ball) * stats.speed * time.delta_secs();
        ball.translation += Vec3::new(forward.x, forward.y, 0.0);
    }
}

fn despawn_after_x(
    time: Res<Time>,
    mut balls: Query<(Entity, &mut DespawnAfter)>,
    mut commands: Commands,
) {
    for (entity, mut ball) in &mut balls {
        ball.0.tick(time.delta());
        if ball.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn cannon_ball_hit(
    balls: Query<(Entity, &Transform, &CanonBall), Without<Enemy>>,
    mut enemies: Query<(&Transform, &mut Health), (With<Enemy>, Without<CanonBall>)>,
    mut commands: Commands,
) {
    for ball in balls {
        for (enemy_transform, mut enemy_health) in &mut enemies {
            if ball
                .1
                .translation
                .distance_squared(enemy_transform.translation)
                < 1000.0
            {
                commands.entity(ball.0).despawn();
                enemy_health.0 -= ball.2.damage as i32;
            }
        }
    }
}

fn flamestrike_hits(
    trigger: On<Insert, FlameStrike>,
    flamestrikes: Query<(&Transform, &FlameStrike)>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
) -> std::result::Result<(), BevyError> {
    let (flame_transform, flame) = flamestrikes.get(trigger.entity)?;
    let r = flame.radius * flame.radius;
    for (enemy_t, mut enemy) in &mut enemies {
        if enemy_t
            .translation
            .distance_squared(flame_transform.translation)
            < r
        {
            enemy.0 -= flame.damage as i32;
        }
    }

    Ok(())
}

fn update_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<WeaponMaterial>>,
    boats: Query<&MeshMaterial2d<WeaponMaterial>>,
) {
    for c in boats.iter() {
        if let Some(m) = materials.get_mut(c.0.id()) {
            m.time = Vec4::new(m.time.x + time.delta_secs(), m.time.y, 0.0, 0.0);
        }
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct WeaponMaterial {
    #[uniform(0)]
    time: Vec4, // the second value decides the type (cannon ball, explosion, etc)
}

const WEAPON_SHADER_PATH: &str = "shaders/weapon.wesl";

impl Material2d for WeaponMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        WEAPON_SHADER_PATH.into()
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
