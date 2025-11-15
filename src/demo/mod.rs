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

pub fn forward_vec(transform: Transform) -> Vec2 {
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin())
}
