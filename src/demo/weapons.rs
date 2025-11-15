use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{
    demo::{enemy::Enemy, forward_vec},
    screens::Screen,
};

pub struct WeaponPlugin;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum WeaponType {
    Canon { _angle: f32, _range: f32 },
}

#[derive(Component)]
pub struct CanonBall {
    pub remaining: Timer,
}

#[derive(Component)]
pub struct WeaponSlots {
    pub left: [Option<(Timer, WeaponType)>; 3],
    pub right: [Option<(Timer, WeaponType)>; 3],
    pub front: Option<(Timer, WeaponType)>,
}

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                cannonball_flight,
                update_time,
                eval_weapons,
                cannonball_despawn,
                cannon_ball_hit,
            )
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_plugins(Material2dPlugin::<WeaponMaterial>::default());
    }
}

fn eval_weapons(
    time: Res<Time>,
    mut weapon_holders: Query<(&mut WeaponSlots, &Transform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WeaponMaterial>>,
) {
    for (mut weapon_holder, transform) in &mut weapon_holders {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
        let forward = Vec2::new(angle.cos(), angle.sin());

        for (i, left_slot) in weapon_holder.left.iter_mut().enumerate() {
            if let Some(left_slot) = left_slot {
                left_slot.0.tick(time.delta());
                if left_slot.0.is_finished() {
                    let weapon_transform = left_weapon_transform(transform, forward, angle, i);

                    commands.spawn(projectile(
                        weapon_transform,
                        Timer::from_seconds(3.0, TimerMode::Once),
                        &mut meshes,
                        &mut materials,
                    ));
                }
            }
        }
        for (i, right_slot) in weapon_holder.right.iter_mut().enumerate() {
            if let Some(right_slot) = right_slot {
                right_slot.0.tick(time.delta());
                if right_slot.0.is_finished() {
                    let weapon_transform = right_weapon_transform(transform, forward, angle, i);
                    commands.spawn(projectile(
                        weapon_transform,
                        Timer::from_seconds(3.0, TimerMode::Once),
                        &mut meshes,
                        &mut materials,
                    ));
                }
            }
        }

        if let Some(front) = &mut weapon_holder.front {
            front.0.tick(time.delta());
            if front.0.is_finished() {
                let weapon_position = transform.translation.xy() + forward * 100.0;
                commands.spawn(projectile(
                    Transform::from_translation(Vec3::new(
                        weapon_position.x,
                        weapon_position.y,
                        0.0,
                    ))
                    .with_rotation(Quat::from_axis_angle(
                        Vec3::Z,
                        angle - std::f32::consts::FRAC_PI_2,
                    )),
                    Timer::from_seconds(3.0, TimerMode::Once),
                    &mut meshes,
                    &mut materials,
                ));
            }
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

fn projectile(
    transform: Transform,
    remaining: Timer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<WeaponMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Rectangle::new(30.0, 30.0));
    let material = materials.add(WeaponMaterial { time: Vec4::ZERO });
    (
        CanonBall { remaining },
        transform,
        Mesh2d(mesh),
        MeshMaterial2d(material),
    )
}

fn cannonball_flight(mut balls: Query<&mut Transform, With<CanonBall>>, time: Res<Time>) {
    for mut ball in &mut balls {
        let forward = forward_vec(*ball) * 300.0 * time.delta_secs();
        ball.translation += Vec3::new(forward.x, forward.y, 0.0);
    }
}
fn cannonball_despawn(
    time: Res<Time>,
    mut balls: Query<(Entity, &mut CanonBall)>,
    mut commands: Commands,
) {
    for (entity, mut ball) in &mut balls {
        ball.remaining.tick(time.delta());
        if ball.remaining.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn cannon_ball_hit(
    balls: Query<(Entity, &Transform), (With<CanonBall>, Without<Enemy>)>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<CanonBall>)>,
    mut commands: Commands,
) {
    for ball in balls {
        for enemy in enemies {
            if ball.1.translation.distance_squared(enemy.1.translation) < 1000.0 {
                commands.entity(ball.0).despawn();
                commands.entity(enemy.0).despawn();
            }
        }
    }
}

fn update_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<WeaponMaterial>>,
    boats: Query<&MeshMaterial2d<WeaponMaterial>>,
) {
    for c in boats.iter() {
        if let Some(m) = materials.get_mut(c.0.id()) {
            m.time = Vec4::new(m.time.x + time.delta_secs(), 0.0, 0.0, 0.0);
        }
    }
}
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct WeaponMaterial {
    #[uniform(0)]
    time: Vec4,
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
