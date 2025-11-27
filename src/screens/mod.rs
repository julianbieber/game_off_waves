//! The game's main screen states and transitions between them.

mod end;
mod gameplay;
mod loading;
mod splash;
mod title;

use bevy::prelude::*;

use crate::screens::end::EndPlugin;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        gameplay::plugin,
        loading::plugin,
        splash::plugin,
        title::plugin,
        EndPlugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Loading,
    Gameplay,
    YouDied,
    Shop,
}
