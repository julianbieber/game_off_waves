//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use std::f32;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    demo::{player::Player, terrain::waves::Waves},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_movement, rotate_forward, apply_waves)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    pub intent: f32,
    pub rotation_intent: f32,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: 0.0,
            rotation_intent: 0.0,
            max_speed: 400.0,
        }
    }
}

fn apply_movement(
    _time: Res<Time>,
    mut movement_query: Query<(&MovementController, &Transform, Forces)>,
) {
    for (controller, transform, mut forces) in &mut movement_query {
        forces.apply_angular_impulse(controller.rotation_intent * 600.0);
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + f32::consts::FRAC_PI_2;
        let forward = Vec2::new(angle.cos(), angle.sin());

        let new_force = forward * controller.intent * 300.0; //* time.delta_secs();

        forces.apply_force(new_force);
    }
}

fn apply_waves(
    time: Res<Time>,
    mut movement_query: Query<(&Transform, Forces)>,
    waves: Query<&Waves>,
) -> std::result::Result<(), BevyError> {
    let waves = waves.single()?;

    for (transform, mut forces) in &mut movement_query {
        let (wave_dir, _wave_height, up) =
            waves.wave_height(transform.translation.xy(), time.elapsed_secs());
        if wave_dir.is_nan() {
            continue;
        }

        if up {
            forces.apply_force(-wave_dir * 1000.0);
        } else {
            forces.apply_force(wave_dir * 500.0);
        }
    }
    Ok(())
}

fn rotate_forward(
    mut velocities: Query<(&Transform, Forces), (With<MovementController>, With<Player>)>,
) {
    for (transform, mut forces) in &mut velocities {
        let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + f32::consts::FRAC_PI_2;
        let forward = Vec2::new(angle.cos(), angle.sin());
        let backward = -forward;
        let velo_to_forward = forces.linear_velocity().normalize().dot(forward);
        let velo_to_backward = forces.linear_velocity().normalize().dot(backward);

        if velo_to_backward < velo_to_forward {
            *forces.linear_velocity_mut() = forward * forces.linear_velocity().length();
        } else {
            *forces.linear_velocity_mut() = backward * forces.linear_velocity().length();
        }
    }
}
