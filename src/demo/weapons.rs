use std::time::Duration;

use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

use crate::{demo::forward_vec, screens::Screen};

pub struct WeaponPlugin;

#[derive(Clone, Copy)]
pub enum WeaponType {
    Canon {
        delay: Duration,
        angle: f32,
        range: f32,
    },
}

#[derive(Component)]
pub struct CanonBall {
    remaining: Timer,
}

#[derive(Component)]
pub struct WeaponSlots {
    left: [Option<WeaponType>; 3],
    right: [Option<WeaponType>; 3],
    front: Option<WeaponType>,
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
            )
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_plugins(Material2dPlugin::<WeaponMaterial>::default());
    }
}

fn eval_weapons(
    _time: Res<Time>,
    mut weapon_holders: Query<(&WeaponSlots, &Transform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WeaponMaterial>>,
) {
    for (weapon_holder, transform) in &mut weapon_holders {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
        let forward = Vec2::new(angle.cos(), angle.sin());
        let left = Vec2::new(
            (angle - std::f32::consts::FRAC_PI_2).cos(),
            (angle - std::f32::consts::FRAC_PI_2).sin(),
        );
        let right = Vec2::new(
            (angle + std::f32::consts::FRAC_PI_2).cos(),
            (angle + std::f32::consts::FRAC_PI_2).sin(),
        );

        for (i, left_slot) in weapon_holder.left.iter().enumerate() {
            if let Some(_left_slot) = left_slot {
                let i = i as f32 - 1.0;
                let weapon_position = transform.translation.xy() + left + i * forward;

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
        for (i, right_slot) in weapon_holder.right.iter().enumerate() {
            if let Some(_right_slot) = right_slot {
                let i = i as f32 - 1.0;
                let weapon_position = transform.translation.xy() + right + i * forward;
                commands.spawn(projectile(
                    Transform::from_translation(Vec3::new(
                        weapon_position.x,
                        weapon_position.y,
                        0.0,
                    ))
                    .with_rotation(Quat::from_axis_angle(
                        Vec3::Z,
                        angle + std::f32::consts::FRAC_PI_2,
                    )),
                    Timer::from_seconds(3.0, TimerMode::Once),
                    &mut meshes,
                    &mut materials,
                ));
            }
        }

        if let Some(_front) = weapon_holder.front {
            let weapon_position = transform.translation.xy() + forward;
            commands.spawn(projectile(
                Transform::from_translation(Vec3::new(weapon_position.x, weapon_position.y, 0.0))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, angle)),
                Timer::from_seconds(3.0, TimerMode::Once),
                &mut meshes,
                &mut materials,
            ));
        }
    }
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

fn cannonball_flight(mut balls: Query<&mut Transform, With<CanonBall>>) {
    for mut ball in &mut balls {
        let forward = forward_vec(*ball);
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
