//! The screen shown when the player dies, with the run's score.

use bevy::prelude::*;

use crate::{demo::player::EnemiesKilled, screens::Screen, theme::widget};

pub struct EndPlugin;

impl Plugin for EndPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::YouDied), spawn_end_screen);
    }
}

fn spawn_end_screen(mut commands: Commands, killed: Res<EnemiesKilled>) {
    let killed = killed.amount;
    commands.spawn_scene(bsn! {
        @widget::ui_root("End Screen")
        ~{DespawnOnExit(Screen::YouDied)}
        Children [ @widget::label(format!("Score: {killed}")) ]
    });
}
