//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use avian2d::prelude::PhysicsLayer;
use bevy::prelude::*;

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

pub fn angle_between(_base: Transform, _point: Vec2) -> f32 {
    0.0
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
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

        assert_eq!(angle_between(base_pointing_y, Vec2::Y), 0.0);
        assert_eq!(angle_between(base_pointing_x, Vec2::X), 0.0);
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
