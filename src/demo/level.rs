//! Spawn the main level.

use bevy::prelude::*;

use crate::{
    demo::player::{BoatMaterial, player},
    screens::Screen,
};

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    mut materials: ResMut<Assets<BoatMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![player(400.0, &mut meshes, &mut materials),],
        // TODO attach the terrain to the level
    ));
}
