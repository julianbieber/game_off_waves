//! Hover and press feedback for buttons: a background colour per state, and a sound on hover and on click.

use bevy::{picking::hover::Hovered, prelude::*, ui::Pressed, ui_widgets::Button};

use crate::{asset_tracking::LoadResource, audio::sound_effect};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply_interaction_palette);

    app.load_resource::<InteractionAssets>();
    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
}

/// Palette for widget interactions. Add this to a [`Button`] to change its
/// [`BackgroundColor`] based on whether it is hovered or pressed.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
#[require(Hovered)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette(
    mut palette_query: Query<(
        &Hovered,
        Has<Pressed>,
        &InteractionPalette,
        &mut BackgroundColor,
    )>,
) {
    for (hovered, pressed, palette, mut background) in &mut palette_query {
        let color = if pressed {
            palette.pressed
        } else if hovered.get() {
            palette.hovered
        } else {
            palette.none
        };
        background.set_if_neq(color.into());
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct InteractionAssets {
    #[dependency]
    hover: Handle<AudioSource>,
    #[dependency]
    click: Handle<AudioSource>,
}

impl FromWorld for InteractionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            hover: assets.load("audio/sound_effects/button_hover.ogg"),
            click: assets.load("audio/sound_effects/button_click.ogg"),
        }
    }
}

fn play_on_hover_sound_effect(
    trigger: On<PointerOver>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), With<Button>>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.entity) {
        commands.spawn(sound_effect(interaction_assets.hover.clone()));
    }
}

fn play_on_click_sound_effect(
    trigger: On<PointerClick>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), With<Button>>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.entity) {
        commands.spawn(sound_effect(interaction_assets.click.clone()));
    }
}
