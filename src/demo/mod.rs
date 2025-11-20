//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use avian2d::prelude::PhysicsLayer;
use bevy::{math::ops::atan2, prelude::*};

pub mod enemy;
pub mod level;
mod movement;
pub mod player;
mod terrain;
pub mod weapons;

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Terrain,
    Player,
    Enemy,
}

#[derive(Component)]
pub struct Health(i32);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        movement::plugin,
        player::plugin,
        terrain::TerrainPlugin,
        enemy::EnemyPlugin,
        weapons::WeaponPlugin,
    ));
}

pub fn forward_vec(transform: &Transform) -> Vec2 {
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin())
}

/// should return the value with lower abs
pub fn angle_between(base: &Transform, point: Vec2) -> f32 {
    let b = (point - base.translation.xy()).normalize();
    let a = forward_vec(base);

    atan2(a.x * b.y - a.y * b.x, a.x * b.x + a.y * b.y)
}

mod test {

    #[allow(unused)]
    use bevy::math::Quat;
    #[allow(unused)]
    use bevy::{math::Vec2, transform::components::Transform};

    #[allow(unused)]
    use crate::demo::angle_between;
    #[allow(unused)]
    use crate::demo::forward_vec;

    #[test]
    fn angle_between_test() {
        let base_pointing_y = Transform::IDENTITY;

        let base_pointing_x =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2));

        eq(angle_between(&base_pointing_y, Vec2::Y), 0.0);
        eq(
            angle_between(&base_pointing_y, Vec2::X),
            -std::f32::consts::FRAC_PI_2,
        );
        eq(angle_between(&base_pointing_x, Vec2::X), 0.0);
        eq(
            angle_between(&base_pointing_x, Vec2::Y),
            std::f32::consts::FRAC_PI_2,
        );
    }

    #[allow(unused)]
    fn eq(a: f32, b: f32) {
        dbg!(a, b);
        assert!((a - b).abs() < 0.001);
    }

    #[test]
    fn forward_test() {
        let towards_top = Transform::IDENTITY;
        vec_eq(forward_vec(&towards_top), Vec2::Y);
        let towards_right =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2));
        vec_eq(forward_vec(&towards_right), Vec2::X);
        let towards_left =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
        vec_eq(forward_vec(&towards_left), -Vec2::X);
        let towards_down =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(-std::f32::consts::PI));
        vec_eq(forward_vec(&towards_down), -Vec2::Y);
    }

    #[allow(unused)]
    fn vec_eq(a: Vec2, b: Vec2) {
        dbg!(a, b);
        assert!(a.distance(b) < 0.01);
    }
}
